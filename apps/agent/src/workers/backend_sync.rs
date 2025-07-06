// standard crates
use std::future::Future;
use std::pin::Pin;
use std::cmp::max;
use std::time::Duration;

// internal modules
use crate::auth::{
    token_mngr::TokenManagerExt,
    token::Token,
};
use crate::errors::*;
use crate::mqtt::errors::*;
use crate::mqtt::client::{
    MQTTClient,
    OptionsBuilder,
    ConnectAddress,
    Credentials,
    poll,
};
use crate::mqtt::device::{DeviceExt, SyncDevice};
use crate::sync::syncer::{SyncerExt, SyncEvent, CooldownEnd};
use crate::utils::{calc_exp_backoff, CooldownOptions};

// external crates
use rumqttc::{Event, Incoming, EventLoop};
use tracing::{error, info, debug};
use tokio::sync::watch;
use chrono::{TimeDelta, Utc};

#[derive(Debug, Clone)]
pub struct BackendSyncWorkerOptions {
    pub poll_interval_secs: i64,
    pub mqtt_cooldown: CooldownOptions,
    pub mqtt_enabled: bool,
    pub mqtt_broker_address: ConnectAddress,
}

impl Default for BackendSyncWorkerOptions {
    fn default() -> Self {
        let twelve_hrs = 12 * 60 * 60;
        Self {
            poll_interval_secs: twelve_hrs,
            mqtt_cooldown: CooldownOptions {
                base_secs: 1,
                growth_factor: 2,
                max_secs: twelve_hrs,
            },
            mqtt_enabled: true,
            mqtt_broker_address: ConnectAddress::default(),
        }
    }
}

pub async fn run_backend_sync_worker<
    TokenManagerT: TokenManagerExt,
    SyncerT: SyncerExt,
>(
    device_id: &str,
    options: &BackendSyncWorkerOptions,
    token_mngr: &TokenManagerT,
    syncer: &SyncerT,
    mut shutdown_signal: Pin<Box<impl Future<Output = ()> + Send + 'static>>,
) {
    if options.mqtt_enabled {
        tokio::select! {
            _ = shutdown_signal.as_mut() => {
                info!("Backend sync worker shutdown complete");
            }
            // these don't return but we do need to run them in the background
            _ = run_polling_sync_worker(
                options.poll_interval_secs,
                syncer,
                tokio::time::sleep,
            ) => {}
            _ = run_mqtt_sync_worker(device_id, options, token_mngr, syncer) => {}
        }
    } else {
        tokio::select! {
            _ = shutdown_signal.as_mut() => {
                info!("Backend sync worker shutdown complete");
            }
            // doesn't return but we do need to run it in the background
            _ = run_polling_sync_worker(
                options.poll_interval_secs,
                syncer,
                tokio::time::sleep,
            ) => {}
        }
    }
}

// ================================ POLLING SYNC =================================== //
pub async fn run_polling_sync_worker<F, Fut, SyncerT: SyncerExt>(
    poll_interval_secs: i64,
    syncer: &SyncerT,
    sleep_fn: F, // for testing purposes
) where
    F: Fn(Duration) -> Fut,
    Fut: Future<Output = ()> + Send
{
    // subscribe to syncer events
    let mut syncer_subscriber = syncer.subscribe().await.unwrap_or_else(|e| {
        error!("error subscribing to syncer events: {e:?}");
        // Create a dummy receiver that never sends anything
        watch::channel(SyncEvent::SyncSuccess).1
    });

    // begin by syncing
    let _ = syncer.sync_if_not_in_cooldown().await;

    loop {
        // poll from the last sync attempt, not the current time
        let last_sync_attempted_at = syncer.get_last_sync_attempted_at().await.unwrap_or_default().timestamp();
        let secs_since_last_sync = Utc::now().timestamp() - last_sync_attempted_at;
        let secs_until_next_sync= poll_interval_secs - secs_since_last_sync;

        // wait until the cooldown ends or the poll interval elapses (max of the two)
        let secs_until_cooldown_ends = syncer.get_cooldown_ends_at().await.unwrap_or_default().signed_duration_since(Utc::now()).num_seconds();
        let wait_secs = max(secs_until_next_sync, secs_until_cooldown_ends);

        // log the next scheduled sync time
        let next_sync_at = Utc::now() + TimeDelta::seconds(wait_secs);
        info!("Waiting until {:?} ({:?} seconds) for next *scheduled* device sync", next_sync_at, wait_secs);

        tokio::select! {
            // next scheduled sync
            _ = sleep_fn(Duration::from_secs(wait_secs as u64)) => {
                let _ = syncer.sync_if_not_in_cooldown().await;
            }

            // listen for syncer events from the syncer worker (this device)
            _ = syncer_subscriber.changed() => {
                let syncer_event = syncer_subscriber.borrow().clone();
                // retry synchronization when the cooldown ends from a failed sync
                if let SyncEvent::CooldownEnd(CooldownEnd::FromSyncFailure) = syncer_event {
                    let _ = syncer.sync_if_not_in_cooldown().await;
                } 
            }
        }
    }
}

// ============================= MQTT SYNC LISTENER ================================ //
pub async fn run_mqtt_sync_worker<
    TokenManagerT: TokenManagerExt,
    SyncerT: SyncerExt,
>(
    device_id: &str,
    options: &BackendSyncWorkerOptions,
    token_mngr: &TokenManagerT,
    syncer: &SyncerT,
) -> Result<(), MQTTError> {

    // subscribe to syncer events
    let mut syncer_subscriber = syncer.subscribe().await.unwrap_or_else(|e| {
        error!("error subscribing to syncer events: {e:?}");
        // Create a dummy receiver that never sends anything
        watch::channel(SyncEvent::SyncSuccess).1
    });

    // create the mqtt client
    let (mut mqtt_client, mut eventloop) = init_mqtt_client(
        device_id, token_mngr, options.mqtt_broker_address.clone()
    ).await;

    let mut mqtt_client_err_streak= 0;
    loop {
        tokio::select! {
            // listen for syncer events from the syncer worker (this device)
            _ = syncer_subscriber.changed() => {
                let syncer_event = syncer_subscriber.borrow().clone();
                handle_syncer_event(
                    &syncer_event,
                    device_id,
                    &mqtt_client,
                ).await;
            }

            // listen for sync commands from the backend (via mqtt broker)
            mqtt_result = poll(&mut eventloop) => {
                match mqtt_result {
                    Ok(mqtt_event) => {
                        mqtt_client_err_streak = 0;
                        handle_mqtt_event(&mqtt_event, syncer).await;
                    }
                    Err(e) => {
                        mqtt_client_err_streak += 1;
                        (mqtt_client, eventloop) = handle_mqtt_error(
                            e,
                            device_id,
                            token_mngr,
                            &options.mqtt_broker_address,
                            mqtt_client,
                            eventloop
                        ).await;
                    }
                }
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

pub async fn handle_syncer_event<MQTTClientT: DeviceExt>(
    event: &SyncEvent,
    device_id: &str,
    mqtt_client: &MQTTClientT,
) {
    if !matches!(event, SyncEvent::SyncSuccess) {
        return;
    }

    // whenever the syncer has synced, we need to publish this synchronization to the 
    // backend
    match mqtt_client.publish_device_sync(device_id).await {
        Ok(_) => {
            info!("successfully published device sync to backend");
        }
        Err(e) => {
            error!("error publishing device sync: {e:?}");
        }
    }
}

async fn init_mqtt_client<TokenManagerT: TokenManagerExt>(
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
    let (mqtt_client, eventloop) = MQTTClient::new(&options).await;

    // subscribe to device synchronization updates
    if let Err(e) = mqtt_client.subscribe_device_sync(device_id).await {
        error!("error subscribing to device synchronization updates: {e:?}");
    };

    (mqtt_client, eventloop)
}

pub async fn handle_mqtt_event<SyncerT: SyncerExt>(
    event: &Event,
    syncer: &SyncerT,
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

    let _ = syncer.sync_if_not_in_cooldown().await;
}

pub async fn handle_mqtt_error<TokenManagerT: TokenManagerExt>(
    e: MQTTError,
    device_id: &str,
    token_mngr: &TokenManagerT,
    broker_address: &ConnectAddress,
    mqtt_client: MQTTClient,
    eventloop: EventLoop,
) -> (MQTTClient, EventLoop) {
    // auth error -> refresh token and reinitialize the mqtt client
    if e.is_authentication_error() {
        error!("authentication error while polling backend for sync command via mqtt: {e:?}");
        info!("attempting to refresh token");
        if let Err(e) = token_mngr.refresh_token().await {
            error!("error refreshing token for backend sync worker: {e:?}");
        }
        init_mqtt_client(device_id, token_mngr, broker_address.clone()).await
    }
    // network connection error -> ignore
    else if e.is_network_connection_error() {
        debug!("network connection error while polling backend for sync command via mqtt: {e:?}");
        (mqtt_client, eventloop)
    // other errors -> log
    } else {
        error!("error polling backend for sync command via mqtt: {e:?}");
        (mqtt_client, eventloop)
    }
}