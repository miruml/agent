// standard crates
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use std::cmp::max;

// internal modules
use crate::auth::{
    token_mngr::TokenManagerExt,
    token::Token,
};
use crate::mqtt::errors::*;
use crate::mqtt::client::{
    MQTTClient,
    OptionsBuilder,
    ConnectAddress,
    Credentials,
    poll,
};
use crate::mqtt::device::SyncDevice;
use crate::sync::syncer::Syncer;
use crate::utils::calc_exp_backoff;
use crate::workers::cooldown::CooldownOptions;

// external crates
use chrono::TimeDelta;
use rumqttc::{Event, Incoming, EventLoop};
use tracing::{error, info};

#[derive(Debug, Clone)]
pub struct BackendSyncWorkerOptions {
    pub poll_secs: u32,
    pub sync_cooldown: CooldownOptions,
    pub mqtt_cooldown: CooldownOptions,
    pub mqtt_enabled: bool,
    pub mqtt_broker_address: ConnectAddress,
}

impl Default for BackendSyncWorkerOptions {
    fn default() -> Self {
        Self {
            poll_secs: 12 * 60 * 60, // 12 hours
            sync_cooldown: CooldownOptions {
                base_secs: 15,
                growth_factor: 2,
                max_secs: 12 * 60 * 60, // 12 hours
            },
            mqtt_cooldown: CooldownOptions {
                base_secs: 1,
                growth_factor: 2,
                max_secs: 12 * 60 * 60, // 12 hours
            },
            mqtt_enabled: true,
            mqtt_broker_address: ConnectAddress::default(),
        }
    }
}

pub async fn run_backend_sync_worker<TokenManagerT: TokenManagerExt>(
    device_id: &str,
    options: &BackendSyncWorkerOptions,
    token_mngr: &TokenManagerT,
    syncer: &Syncer,
    mut shutdown_signal: Pin<Box<impl Future<Output = ()> + Send + 'static>>,
) {
    if options.mqtt_enabled {
        tokio::select! {
            _ = shutdown_signal.as_mut() => {
                info!("Backend sync worker shutdown complete");
            }
            // these don't return but we do need to run them in the background
            _ = run_polling_sync(options, syncer) => {}
            _ = run_mqtt_sync(device_id, options, token_mngr, syncer) => {}
        }
    } else {
        tokio::select! {
            _ = shutdown_signal.as_mut() => {
                info!("Backend sync worker shutdown complete");
            }
            // this doesn't return but we do need to run it in the background
            _ = run_polling_sync(options, syncer) => {}
        }
    }
}

// ================================ POLLING SYNC =================================== //
pub async fn run_polling_sync(options: &BackendSyncWorkerOptions, syncer: &Syncer) {
    let mut sync_err_streak= 0;

    loop {
        let cooldown_secs = calc_exp_backoff(
            options.sync_cooldown.base_secs,
            options.sync_cooldown.growth_factor,
            sync_err_streak,
            options.sync_cooldown.max_secs
        );

        // wait using the poll period
        let cooldown_duration = Duration::from_secs(cooldown_secs as u64);
        let poll_duration = Duration::from_secs(options.poll_secs as u64);
        let wait = max(cooldown_duration, poll_duration);
        tokio::time::sleep(wait).await;

        let min_cooldown = TimeDelta::seconds(cooldown_secs);
        match syncer.sync(min_cooldown).await {
            Ok(_) => {
                sync_err_streak = 0;
            }
            Err(e) => {
                error!("error syncing device: {e:?}");
                sync_err_streak += 1;
            }
        }
    }
}

// ============================= MQTT SYNC LISTENER ================================ //
async fn run_mqtt_sync<TokenManagerT: TokenManagerExt>(
    device_id: &str,
    options: &BackendSyncWorkerOptions,
    token_mngr: &TokenManagerT,
    syncer: &Syncer,
) -> Result<(), MQTTError> {

    // create the mqtt client
    let (mqtt_client, mut eventloop) = create_mqtt_client(
        device_id, token_mngr, options.mqtt_broker_address.clone()
    ).await;

    // subscribe to device synchronization updates
    mqtt_client.subscribe_device_sync(device_id).await?;

    // listen for sync commands from the backend
    let mut mqtt_client_err_streak= 0;
    loop {
        let event = poll(&mut eventloop).await;
        match event {
            Ok(event) => {
                mqtt_client_err_streak = 0;
                handle_mqtt_event(
                    device_id,
                    &mqtt_client,
                    &event,
                    syncer,
                    &options.sync_cooldown
                ).await;
            }
            Err(e) => {
                mqtt_client_err_streak += 1;
                handle_mqtt_error(e, token_mngr).await;
            }
        }

        // sleep for the cooldown period to prevent throttling from mqtt errors
        let cooldown_secs = calc_exp_backoff(
            options.mqtt_cooldown.base_secs,
            options.mqtt_cooldown.growth_factor,
            mqtt_client_err_streak,
            options.mqtt_cooldown.max_secs
        );
        let cooldown_duration = Duration::from_secs(cooldown_secs as u64);
        tokio::time::sleep(cooldown_duration).await;
    }
}

async fn create_mqtt_client<TokenManagerT: TokenManagerExt>(
    device_id: &str,
    token_mngr: &TokenManagerT,
    broker_address: ConnectAddress,
) -> (MQTTClient, EventLoop) {
    // update the mqtt password
    let token = match token_mngr.get_token().await {
        Ok(token) => token.token.clone(),
        Err(_) => { Token::default().token }
    };

    // initialize the mqtt options
    let credentials = Credentials {
        username: device_id.to_string(),
        password: token,
    };
    let options = OptionsBuilder::new(credentials)
        .with_connect_address(broker_address)
        .build();

    // create the mqtt client
    MQTTClient::new(&options).await
}

pub async fn handle_mqtt_event(
    device_id: &str,
    mqtt_client: &MQTTClient,
    event: &Event,
    syncer: &Syncer,
    sync_cooldown_opts: &CooldownOptions,
) {

    // ignore non-publish events
    let publish = match event {
        Event::Incoming(Incoming::Publish(publish)) => publish,
        // non-publish events are not sync requests so device is still considered synced
        _ => return,
    };

    // deserialize the sync request
    let is_synced = match serde_json::from_slice::<SyncDevice>(&publish.payload) {
        Ok(sync_req) => sync_req.is_synced,
        Err(e) => {
            error!("error deserializing sync request: {e:?}");
            false
        }
    };
    if is_synced {
        return;
    }

    // sync the device
    let min_cooldown = TimeDelta::seconds(sync_cooldown_opts.base_secs);
    match syncer.sync(min_cooldown).await {
        Ok(_) => {
            if let Err(e) = mqtt_client.publish_device_sync(device_id).await {
                error!("error publishing device sync: {e:?}");
            }
        }
        Err(e) => {
            error!("error syncing device: {e:?}");
        }
    }
}

pub async fn handle_mqtt_error<TokenManagerT: TokenManagerExt>(
    e: MQTTError,
    token_mngr: &TokenManagerT
) {
    match e {
        MQTTError::AuthenticationErr(e) => {
            error!("authentication error while polling backend for sync command via mqtt: {e:?}");
            info!("attempting to refresh token");
            // refresh the token
            if let Err(e) = token_mngr.refresh_token().await {
                error!("error refreshing token for backend sync worker: {e:?}");
            }
        }

        // ignore network connection errors and continue polling
        MQTTError::NetworkConnectionErr(_) => {}

        // log any other errors
        _ => {
            error!("error polling backend for sync command via mqtt: {e:?}");
        }
    }
}