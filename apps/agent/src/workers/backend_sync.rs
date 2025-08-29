// standard crates
use std::cmp::max;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

// internal modules
use crate::authn::{token::Token, token_mngr::TokenManagerExt};
use crate::errors::*;
use crate::models::device::{self, Device, DeviceStatus};
use crate::mqtt::client::{poll, ConnectAddress, Credentials, MQTTClient, OptionsBuilder};
use crate::mqtt::device::{DeviceExt, SyncDevice};
use crate::mqtt::errors::*;
use crate::storage::device::DeviceFile;
use crate::sync::syncer::{CooldownEnd, SyncEvent, SyncerExt};
use crate::utils::{calc_exp_backoff, CooldownOptions};

// external crates
use chrono::{TimeDelta, Utc};
use rumqttc::{ConnectReturnCode, Event, EventLoop, Incoming};
use tokio::sync::watch;
use tracing::{debug, error, info};

#[derive(Debug, Clone)]
pub struct BackendSyncWorkerOptions {
    pub poll_interval_secs: i64,
    pub mqtt_enabled: bool,
    pub mqtt_cooldown: CooldownOptions,
    pub mqtt_broker_address: ConnectAddress,
}

impl Default for BackendSyncWorkerOptions {
    fn default() -> Self {
        let twelve_hrs = 12 * 60 * 60;
        Self {
            poll_interval_secs: twelve_hrs,
            mqtt_enabled: true,
            mqtt_cooldown: CooldownOptions {
                base_secs: 1,
                growth_factor: 2,
                max_secs: twelve_hrs,
            },
            mqtt_broker_address: ConnectAddress::default(),
        }
    }
}

pub async fn run_backend_sync_worker<TokenManagerT: TokenManagerExt, SyncerT: SyncerExt>(
    options: &BackendSyncWorkerOptions,
    token_mngr: &TokenManagerT,
    syncer: &SyncerT,
    device_file: &DeviceFile,
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
                device_file,
                tokio::time::sleep,
            ) => {}
            _ = run_mqtt_sync_worker(options, token_mngr, syncer, device_file) => {}
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
                device_file,
                tokio::time::sleep,
            ) => {}
        }
    }
}

// ================================ POLLING SYNC =================================== //
pub async fn run_polling_sync_worker<F, Fut, SyncerT: SyncerExt>(
    poll_interval_secs: i64,
    syncer: &SyncerT,
    device_file: &DeviceFile,
    sleep_fn: F, // for testing purposes
) where
    F: Fn(Duration) -> Fut,
    Fut: Future<Output = ()> + Send,
{
    info!("Running polling backend sync worker");

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
        let last_attempted_sync_at = syncer
            .get_last_attempted_sync_at()
            .await
            .unwrap_or_default()
            .timestamp();
        let secs_since_last_sync = Utc::now().timestamp() - last_attempted_sync_at;
        let secs_until_next_sync = poll_interval_secs - secs_since_last_sync;

        // wait until the cooldown ends or the poll interval elapses (max of the two)
        let secs_until_cooldown_ends = syncer
            .get_cooldown_ends_at()
            .await
            .unwrap_or_default()
            .signed_duration_since(Utc::now())
            .num_seconds();
        let wait_secs = max(secs_until_next_sync, secs_until_cooldown_ends);

        // log the next scheduled sync time
        let next_sync_at = Utc::now() + TimeDelta::seconds(wait_secs);
        debug!(
            "Waiting until {:?} ({:?} seconds) for next *scheduled* device sync",
            next_sync_at, wait_secs
        );

        tokio::select! {
            // next scheduled sync
            _ = sleep_fn(Duration::from_secs(wait_secs as u64)) => {
                let _ = syncer.sync_if_not_in_cooldown().await;
            }

            // listen for syncer events from the syncer worker (this device)
            _ = syncer_subscriber.changed() => {
                let syncer_event = syncer_subscriber.borrow().clone();

                match &syncer_event {
                    SyncEvent::CooldownEnd(CooldownEnd::FromSyncFailure) => {
                        let _ = syncer.sync_if_not_in_cooldown().await;
                    }
                    SyncEvent::SyncSuccess => {
                        let patch = device::Updates {
                            last_synced_at: Some(Utc::now()),
                            ..device::Updates::empty()
                        };
                        let _ = device_file.patch(patch).await;
                    }
                    _ => {}
                }
            }
        }
    }
}

// ============================= MQTT SYNC LISTENER ================================ //
pub async fn run_mqtt_sync_worker<TokenManagerT: TokenManagerExt, SyncerT: SyncerExt>(
    options: &BackendSyncWorkerOptions,
    token_mngr: &TokenManagerT,
    syncer: &SyncerT,
    device_file: &DeviceFile,
) -> Result<(), MQTTError> {
    info!("Running mqtt backend sync worker");

    // subscribe to syncer events
    let mut syncer_subscriber = syncer.subscribe().await.unwrap_or_else(|e| {
        error!("error subscribing to syncer events: {e:?}");
        // Create a dummy receiver that never sends anything
        watch::channel(SyncEvent::SyncSuccess).1
    });

    let device = device_file
        .read()
        .await
        .unwrap_or_else(|_| Arc::new(device::Device::default()));

    // create the mqtt client
    let (mqtt_client, eventloop) =
        init_mqtt_client(
            &device.id,
            &device.session_id,
            token_mngr,
            options.mqtt_broker_address.clone(),
        )
        .await;

    let mut mqtt_state = MqttState {
        mqtt_client,
        eventloop,
        err_streak: 0,
    };

    loop {
        tokio::select! {
            // listen for syncer events from the syncer worker (this device)
            _ = syncer_subscriber.changed() => {
                let syncer_event = syncer_subscriber.borrow().clone();
                handle_syncer_event(
                    &syncer_event,
                    &device.id,
                    &mqtt_state.mqtt_client,
                ).await;
            }

            // listen for sync commands from the backend (via mqtt broker)
            mqtt_result = poll(&mut mqtt_state.eventloop) => {
                match mqtt_result {
                    Ok(mqtt_event) => {
                        mqtt_state.err_streak = handle_mqtt_event(
                            &mqtt_event,
                            syncer,
                            device_file,
                        ).await;
                    }
                    Err(e) => {
                        mqtt_state = handle_mqtt_error(
                            mqtt_state,
                            e,
                            &device,
                            token_mngr,
                            &options.mqtt_broker_address,
                            device_file,
                        ).await;
                    }
                }
            }
        }

        // sleep for the cooldown period to prevent throttling from mqtt errors
        let cooldown_secs = calc_exp_backoff(
            options.mqtt_cooldown.base_secs,
            options.mqtt_cooldown.growth_factor,
            mqtt_state.err_streak,
            options.mqtt_cooldown.max_secs,
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
    device_session_id: &str,
    token_mngr: &TokenManagerT,
    broker_address: ConnectAddress,
) -> (MQTTClient, EventLoop) {
    // update the mqtt password
    let token = match token_mngr.get_token().await {
        Ok(token) => token.token.clone(),
        Err(_) => Token::default().token,
    };

    // initialize the mqtt options
    let credentials = Credentials {
        username: device_session_id.to_string(),
        password: token,
    };
    let options = OptionsBuilder::new(credentials)
        .with_connect_address(broker_address)
        .with_client_id(device_id.to_string())
        .build();

    // create the mqtt client
    let (mqtt_client, eventloop) = MQTTClient::new(&options).await;

    // subscribe to device synchronization updates
    if let Err(e) = mqtt_client.subscribe_device_sync(device_id).await {
        error!("error subscribing to device synchronization updates: {e:?}");
    };

    (mqtt_client, eventloop)
}

type ErrStreak = u32;

pub async fn handle_mqtt_event<SyncerT: SyncerExt>(
    event: &Event,
    syncer: &SyncerT,
    device_file: &DeviceFile,
) -> ErrStreak {
    let err_streak = 0;

    match event {
        // sync the device if the payload is a sync request
        Event::Incoming(Incoming::Publish(publish)) => {
            let is_synced = match serde_json::from_slice::<SyncDevice>(&publish.payload) {
                Ok(sync_req) => sync_req.is_synced,
                Err(e) => {
                    error!("error deserializing sync request: {e:?}");
                    false
                }
            };
            if is_synced {
                return err_streak;
            }

            let _ = syncer.sync_if_not_in_cooldown().await;
        }
        // update the device connection status on successful connections
        Event::Incoming(Incoming::ConnAck(connack)) => {
            if connack.code != ConnectReturnCode::Success {
                return err_streak;
            }
            info!("Established connection to mqtt broker");
            let _ = device_file.patch(device::Updates::connected()).await;
        }
        // update the device connection status on successful disconnections
        Event::Incoming(Incoming::Disconnect) => {
            info!("Disconnected from mqtt broker");
            let _ = device_file.patch(device::Updates::disconnected()).await;
        }
        _ => {}
    }

    err_streak
}

pub struct MqttState {
    pub mqtt_client: MQTTClient,
    pub eventloop: EventLoop,
    pub err_streak: ErrStreak,
}

pub async fn handle_mqtt_error<TokenManagerT: TokenManagerExt>(
    mut mqtt_state: MqttState,
    e: MQTTError,
    device: &Device,
    token_mngr: &TokenManagerT,
    broker_address: &ConnectAddress,
    device_file: &DeviceFile,
) -> MqttState {
    mqtt_state.err_streak = if e.is_network_connection_error() {
        // don't increment the error streak on network connection errors
        mqtt_state.err_streak
    } else {
        mqtt_state.err_streak + 1
    };

    // update the device to be offline
    match device_file.read().await {
        Ok(device) => {
            if device.status == DeviceStatus::Online {
                let _ = device_file.patch(device::Updates::disconnected()).await;
            }
        }
        Err(_) => {
            let _ = device_file.patch(device::Updates::disconnected()).await;
        }
    }

    // auth error -> refresh token and reinitialize the mqtt client
    if e.is_authentication_error() {
        error!("authentication error while polling backend for sync command via mqtt: {e:?}");
        info!("attempting to refresh token");
        if let Err(e) = token_mngr.refresh_token().await {
            error!("error refreshing token for backend sync worker: {e:?}");
        }
        let (mqtt_client, eventloop) =
            init_mqtt_client(
                &device.id,
                &device.session_id,
                token_mngr,
                broker_address.clone(),
            )
                .await;
        mqtt_state.mqtt_client = mqtt_client;
        mqtt_state.eventloop = eventloop;
        mqtt_state
    }
    // network connection error -> ignore
    else if e.is_network_connection_error() {
        debug!("network connection error while polling backend for sync command via mqtt: {e:?}");
        mqtt_state
    // other errors -> log
    } else {
        error!("error polling backend for sync command via mqtt: {e:?}");
        mqtt_state
    }
}
