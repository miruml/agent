// standard crates
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

// internal modules
use crate::auth::{
    token_mngr::TokenManager,
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

// external crates
use chrono::TimeDelta;
use rumqttc::{Event, Incoming, EventLoop};
use tracing::{error, info};


#[derive(Debug, Clone)]
pub struct CooldownOptions {
    pub base_secs: u32,
    pub growth_factor: u32,
    pub max_secs: u32,
}

impl Default for CooldownOptions {
    fn default() -> Self {
        Self {
            base_secs: 15,
            growth_factor: 2,
            max_secs: 12 * 60 * 60, // 12 hours
        }
    }
}

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
            sync_cooldown: CooldownOptions::default(),
            mqtt_cooldown: CooldownOptions {
                base_secs: 1,
                ..Default::default()
            },
            mqtt_enabled: true,
            mqtt_broker_address: ConnectAddress::default(),
        }
    }
}

pub async fn run_backend_sync_worker(
    device_id: String,
    options: BackendSyncWorkerOptions,
    token_mngr: &TokenManager,
    syncer: &Syncer,
    mut shutdown_signal: Pin<Box<impl Future<Output = ()> + Send + 'static>>,
) {
    let mut sync_err_streak= 0;

    loop {
        let wait = if sync_err_streak > 0 {
            // wait using the cooldown period
            let cooldown_secs = calc_exp_backoff(
                options.sync_cooldown.base_secs,
                options.sync_cooldown.growth_factor,
                sync_err_streak,
                options.sync_cooldown.max_secs
            );
            Duration::from_secs(cooldown_secs as u64)
        } else {
            // wait using the poll period
            Duration::from_secs(options.poll_secs as u64)
        };

        // ----------------------------- WITHOUT MQTT ------------------------------ //
        if !options.mqtt_enabled {
            tokio::select! {
                _ = shutdown_signal.as_mut() => {
                    info!("Backend sync worker shutdown complete");
                    return;
                }
                _ = tokio::time::sleep(wait) => {}
            }
        // ------------------------------- WITH MQTT ------------------------------- //
        } else {
            tokio::select! {
                _ = shutdown_signal.as_mut() => {
                    info!("Backend sync worker shutdown complete");
                    return;
                }
                _ = tokio::time::sleep(wait) => {}
                _ = listen_for_sync_command(&device_id, token_mngr, &options) => {}
            }
        }

        // Never sync the device in less time than the base cooldown period. This is to
        // prevent the mqtt client from sending sync commands (and / or duplicates) too
        // frequently. This is a separate mechanism from the cooldown period from the
        // sync error streak, as the mqtt client could invoke a sync command sooner than
        // the error cooldown period, which we will allow (as long it is no shorter than
        // the base of course)
        match syncer.sync(TimeDelta::seconds(options.sync_cooldown.max_secs as i64)).await {
            Ok(_) => { sync_err_streak = 0; }
            Err(e) => {
                error!("error syncing device: {e:?}");
                sync_err_streak += 1;
            }
        }
    }
}

// =============================== MQTT LISTENER =================================== //
async fn listen_for_sync_command(
    device_id: &str,
    token_mngr: &TokenManager,
    options: &BackendSyncWorkerOptions,
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
                let is_synced = handle_mqtt_event(&event).await;
                if !is_synced {
                    return Ok(());
                }
            }
            Err(e) => {
                mqtt_client_err_streak += 1;
                handle_listener_error(e, token_mngr).await;
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

async fn create_mqtt_client(
    device_id: &str,
    token_mngr: &TokenManager,
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

type IsSynced = bool;

async fn handle_mqtt_event(event: &Event) -> IsSynced {

    // ignore non-publish events
    let publish = match event {
        Event::Incoming(Incoming::Publish(publish)) => publish,
        // non-publish events are not sync requests so device is still considered synced
        _ => return true,
    };

    // deserialize the sync request
    let is_synced = match serde_json::from_slice::<SyncDevice>(&publish.payload) {
        Ok(sync_req) => sync_req.is_synced,
        Err(e) => {
            error!("error deserializing sync request: {e:?}");
            false
        }
    };
    is_synced
}

async fn handle_listener_error(e: MQTTError, token_mngr: &TokenManager) {
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