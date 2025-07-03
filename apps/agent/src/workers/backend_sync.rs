// standard crates
use std::future::Future;
use std::time::Duration;
use std::cmp::{min, max};

// internal modules
use crate::auth::{
    token_mngr::TokenManager,
    token::Token,
};
use crate::mqtt::errors::*;
use crate::mqtt::client::{
    MQTTClient,
    OptionsBuilder,
    Options,
    ConnectAddress,
    Credentials,
    poll,
};
use crate::mqtt::device::SyncDevice;
use crate::sync::syncer::Syncer;
use crate::utils::as_duration;

// external crates
use chrono::{TimeDelta, Utc};
use rumqttc::{Event, Incoming, Publish};
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
    pub sync_cooldown: CooldownOptions,
    pub mqtt_cooldown: CooldownOptions,
    pub poll_interval: Duration,
    pub mqtt_enabled: bool,
    pub mqtt_broker_address: ConnectAddress,
}

impl Default for BackendSyncWorkerOptions {
    fn default() -> Self {
        Self {
            sync_cooldown: CooldownOptions::default(),
            mqtt_cooldown: CooldownOptions {
                base_secs: 1,
                ..Default::default()
            },
            poll_interval: Duration::from_secs(12 * 60 * 60), // 12 hours
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
    shutdown_signal: impl Future<Output = ()> + Send + 'static,
) {
    let mut shutdown = Box::pin(shutdown_signal);

    // initialize the mqtt options
    let credentials = Credentials {
        username: device_id.to_string(),
        password: "".to_string(),
    };
    let mut mqtt_options = OptionsBuilder::new(credentials)
        .with_connect_address(options.mqtt_broker_address.clone())
        .build();

    let mut mqtt_cooldown_secs = options.mqtt_cooldown.base_secs;
    let mut sync_cooldown_secs = options.sync_cooldown.base_secs;

    loop {
        let duration_until_next_sync = calc_duration_until_next_sync(
            syncer, sync_cooldown_secs, options.sync_cooldown.base_secs
        ).await;

        // ----------------------------- WITHOUT MQTT ----------------------------- //
        if !options.mqtt_enabled {
            tokio::select! {
                _ = shutdown.as_mut() => {
                    info!("Backend sync worker shutdown complete");
                    return;
                }

                _ = tokio::time::sleep(duration_until_next_sync) => {}
            }

        // ------------------------------ WITH MQTT ------------------------------- //
        } else {
            // update the mqtt password
            let token = match token_mngr.get_token().await {
                Ok(token) => token.token.clone(),
                Err(_) => { Token::default().token }
            };
            mqtt_options.credentials.password = token;

            tokio::select! {
                _ = shutdown.as_mut() => {
                    info!("Backend sync worker shutdown complete");
                    return;
                }

                result = listen_for_backend_sync(
                    &mqtt_options,
                    device_id.clone(),
                    TimeDelta::seconds(mqtt_cooldown_secs as i64)
                ) => {
                    match result {
                        Ok(_) => {
                            // reset the cooldown
                            mqtt_cooldown_secs = options.mqtt_cooldown.base_secs;
                        }
                        Err(e) => {
                            handle_listener_error(e, token_mngr).await;
                            mqtt_cooldown_secs = calc_next_cooldown(
                                mqtt_cooldown_secs,
                                options.mqtt_cooldown.growth_factor,
                                options.mqtt_cooldown.max_secs
                            );
                            continue;
                        }
                    }
                },

                _ = tokio::time::sleep(duration_until_next_sync) => {}
            }
        }

        // sync the device
        match syncer.sync(TimeDelta::seconds(sync_cooldown_secs as i64)).await {
            Ok(_) => {
                // reset the cooldown
                sync_cooldown_secs = options.sync_cooldown.base_secs;
            }
            Err(e) => {
                error!("error syncing device: {e:?}");
                sync_cooldown_secs = calc_next_cooldown(
                    sync_cooldown_secs,
                    options.sync_cooldown.growth_factor,
                    options.sync_cooldown.max_secs
                );
            }
        }
    }
}

pub async fn calc_duration_until_next_sync(
    syncer: &Syncer,
    cooldown_secs: u32,
    base_secs: u32,
) -> Duration {

    let last_synced_at = syncer.get_last_synced_at().await.unwrap_or(
        Utc::now()
    ).timestamp();
    let secs_until_next_sync= max(
        last_synced_at + cooldown_secs as i64,
        base_secs as i64
    );
    Duration::from_secs(secs_until_next_sync as u64)
}

async fn handle_listener_error(
    e: MQTTError,
    token_mngr: &TokenManager,
) {
    match e {
        MQTTError::AuthenticationErr(_) => {
            // refresh the token
            if let Err(e) = token_mngr.refresh_token().await {
                error!("error refreshing token for backend sync worker: {e:?}");
            }
        }
        _ => {
            error!("error listening for backend sync command via mqtt: {e:?}");
        }
    }
}

// =============================== MQTT LISTENER =================================== //
async fn listen_for_backend_sync(
    mqtt_options: &Options,
    device_id: String,
    cooldown: TimeDelta,
) -> Result<(), MQTTError> {

    // sleep for the cooldown period to prevent throttling
    let sleep_duration = as_duration(cooldown);
    tokio::time::sleep(sleep_duration).await;

    // create the mqtt client and subscribe to the device sync topic
    let (mqtt_client, mut eventloop) = MQTTClient::new(mqtt_options).await;
    mqtt_client.subscribe_device_sync(&device_id).await.unwrap();

    let mut is_synced;
    loop {
        let event = poll(&mut eventloop).await;
        match event {
            Ok(event) => {
                if let Event::Incoming(Incoming::Publish(publish)) = event {
                    is_synced = handle_mqtt_publish_event(&publish).await.unwrap_or(false);
                    // only return if the device is not synced
                    if !is_synced {
                        return Ok(());
                    }
                }
            }
            Err(e) => {
                let _ = handle_mqtt_error(e).await;
            }
        }
    }
}

type IsSynced = bool;

async fn handle_mqtt_publish_event(publish: &Publish) -> Result<IsSynced, MQTTError> {
    let is_synced = match serde_json::from_slice::<SyncDevice>(&publish.payload) {
        Ok(sync_req) => sync_req.is_synced,
        Err(e) => {
            error!("error deserializing sync request: {e:?}");
            false
        }
    };
    Ok(is_synced)
}

async fn handle_mqtt_error(e: MQTTError) -> Result<(), MQTTError> {
    match e {
        MQTTError::AuthenticationErr(e) => {
            Err(MQTTError::AuthenticationErr(e))
        }
        MQTTError::NetworkConnectionErr(_) => {
            // ignore network connection errors and continue polling
            Ok(())
        }
        _ => {
            error!("error polling backend for sync command via mqtt: {e:?}");
            Ok(())
        }
    }
}

fn calc_next_cooldown(
    cur: u32,
    growth_factor: u32,
    max: u32,
) -> u32 {
    let next = cur.saturating_mul(growth_factor);
    min(next, max)
}