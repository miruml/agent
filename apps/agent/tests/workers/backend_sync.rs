// standard crates
use std::{cmp::max, path::PathBuf, sync::Arc};

// internal crates
use config_agent::auth::token::Token;
use config_agent::logs::*;
use config_agent::mqtt::{
    device::SyncDevice,
    errors::*,
};
use config_agent::sync::errors::{SyncErr, MockErr as SyncMockErr};
use config_agent::workers::{
    backend_sync::{
        BackendSyncWorkerOptions,
        run_polling_sync,
        run_mqtt_sync,
        handle_mqtt_event,
        handle_mqtt_error,
    },
    cooldown::CooldownOptions,
};
use config_agent::utils::calc_exp_backoff;

use crate::auth::mock::MockTokenManager;
use crate::mock::SleepController;
use crate::mqtt::mock::MockDeviceClient;
use crate::sync::mock::MockSyncer;

// external crates
use chrono::{DateTime, TimeDelta, Utc};
use rumqttc::{Event, Incoming, Publish, QoS};

pub mod run_polling_sync {
    use super::*;

    #[tokio::test]
    async fn success_not_in_cooldown() {
        let options = BackendSyncWorkerOptions::default();
        let syncer = Arc::new(MockSyncer::default());
        let sleep_ctrl = Arc::new(SleepController::new());

        let options_for_spawn = options.clone();
        let syncer_for_spawn = syncer.clone();
        let sleep_ctrl_for_spawn = sleep_ctrl.clone();
        let _handle = tokio::spawn(async move {
            run_polling_sync(
                &options_for_spawn,
                syncer_for_spawn.as_ref(),
                sleep_ctrl_for_spawn.sleep_fn(),
            ).await;
        });

        // these sleeps should wait for the polling interval less the last sync time (which is 0)
        for i in 0..10 {
            sleep_ctrl.await_sleep().await;
            let last_sleep = sleep_ctrl.get_last_sleep().unwrap();
            assert_eq!(last_sleep.as_secs(), options.poll_secs as u64);
            assert_eq!(syncer.num_sync_calls(), i + 1);

            // reset the last sync attempted at
            syncer.set_last_sync_attempted_at(DateTime::<Utc>::UNIX_EPOCH);

            sleep_ctrl.release().await;
        }

        // these sleeps should wait for the polling interval less the last sync time (which is 0)
        for _ in 0..10 {
            sleep_ctrl.await_sleep().await;
            let last_sleep = sleep_ctrl.get_last_sleep().unwrap();
            assert_eq!(last_sleep.as_secs(), options.poll_secs as u64);
            // only syncs once *in this loop* since the syncer will be in cooldown after
            // first sync (so it syncs the 11th time first iteration and then doesn't
            // sync for the other iterations)
            assert_eq!(syncer.num_sync_calls(), 11);
            sleep_ctrl.release().await;
        }
    }

    #[tokio::test]
    async fn success_syncer_in_cooldown() {
        let options = BackendSyncWorkerOptions {
            poll_secs: 60,
            sync_cooldown: CooldownOptions {
                base_secs: 60,
                growth_factor: 2,
                max_secs: 60 * 60 * 24, // 1 day
            },
            ..Default::default()
        };
        let syncer = Arc::new(MockSyncer::default());
        syncer.set_last_sync_attempted_at(Utc::now() - TimeDelta::seconds(30));
        let sleep_ctrl = Arc::new(SleepController::new());

        let options_for_spawn = options.clone();
        let syncer_for_spawn = syncer.clone();
        let sleep_ctrl_for_spawn = sleep_ctrl.clone();
        let _handle = tokio::spawn(async move {
            run_polling_sync(
                &options_for_spawn,
                syncer_for_spawn.as_ref(),
                sleep_ctrl_for_spawn.sleep_fn(),
            ).await;
        });

        // should wait for 30 seconds since the polling interval is 60 seconds and the
        // last sync was 30 seconds ago
        sleep_ctrl.await_sleep().await;
        let last_sleep = sleep_ctrl.get_last_sleep().unwrap();
        assert_eq!(last_sleep.as_secs(), 30);
        assert_eq!(syncer.num_sync_calls(), 0);
        sleep_ctrl.release().await;
    }

    #[tokio::test]
    async fn network_error() {
        let options = BackendSyncWorkerOptions {
            poll_secs: 3,
            sync_cooldown: CooldownOptions {
                base_secs: 2,
                growth_factor: 2,
                max_secs: 60 * 60 * 24, // 1 day
            },
            ..Default::default()
        };

        let syncer = Arc::new(MockSyncer::default());
        syncer.set_sync(|| Err(SyncErr::MockErr(Box::new(SyncMockErr {
            is_network_connection_error: true,
        }))));
        let sleep_ctrl = Arc::new(SleepController::new());

        let options_for_spawn = options.clone();
        let syncer_for_spawn = syncer.clone();
        let sleep_ctrl_for_spawn = sleep_ctrl.clone();
        let _handle = tokio::spawn(async move {
            run_polling_sync(
                &options_for_spawn,
                syncer_for_spawn.as_ref(),
                sleep_ctrl_for_spawn.sleep_fn(),
            ).await;
        });

        // these sleeps should still wait for the polling interval
        let mut expected_num_syncs = 0;
        for _ in 0..10 {
            sleep_ctrl.await_sleep().await;
            let last_sleep = sleep_ctrl.get_last_sleep().unwrap();
            assert_eq!(last_sleep.as_secs(), options.poll_secs as u64);
            expected_num_syncs += 1;
            assert_eq!(syncer.num_sync_calls(), expected_num_syncs);
            sleep_ctrl.release().await;
        }
    }

    #[tokio::test]
    async fn non_network_error() {
        let options = BackendSyncWorkerOptions {
            poll_secs: 3,
            sync_cooldown: CooldownOptions {
                base_secs: 2,
                growth_factor: 2,
                max_secs: 60 * 60 * 24, // 1 day
            },
            ..Default::default()
        };
        let syncer = Arc::new(MockSyncer::default());
        syncer.set_sync(|| Err(SyncErr::MockErr(Box::new(SyncMockErr {
            is_network_connection_error: false,
        }))));
        let sleep_ctrl = Arc::new(SleepController::new());

        let options_for_spawn = options.clone();
        let syncer_for_spawn = syncer.clone();
        let sleep_ctrl_for_spawn = sleep_ctrl.clone();
        let _handle = tokio::spawn(async move {
            run_polling_sync(
                &options_for_spawn,
                syncer_for_spawn.as_ref(),
                sleep_ctrl_for_spawn.sleep_fn(),
            ).await;
        });

        // these sleeps should wait for the maximum of the polling interval and the
        // sync cooldown
        for i in 0..30 {

            let cooldown_secs = calc_exp_backoff(
                options.sync_cooldown.base_secs,
                options.sync_cooldown.growth_factor,
                i + 1,
                options.sync_cooldown.max_secs
            );
            let expected_sleep_secs = max(
                options.poll_secs as u64,
                cooldown_secs as u64,
            );

            sleep_ctrl.await_sleep().await;
            let last_sleep = sleep_ctrl.get_last_sleep().unwrap();
            assert_eq!(expected_sleep_secs, last_sleep.as_secs());
            assert_eq!(syncer.num_sync_calls(), (i + 1) as usize);
            sleep_ctrl.release().await;
        }
    }

    #[tokio::test]
    async fn error_recovery() {
        let options = BackendSyncWorkerOptions {
            poll_secs: 3,
            sync_cooldown: CooldownOptions {
                base_secs: 2,
                growth_factor: 2,
                max_secs: 60 * 60 * 24, // 1 day
            },
            ..Default::default()
        };
        let syncer = Arc::new(MockSyncer::default());
        syncer.set_sync(|| Err(SyncErr::MockErr(Box::new(SyncMockErr {
            is_network_connection_error: false,
        }))));
        let sleep_ctrl = Arc::new(SleepController::new());

        let options_for_spawn = options.clone();
        let syncer_for_spawn = syncer.clone();
        let sleep_ctrl_for_spawn = sleep_ctrl.clone();
        let _handle = tokio::spawn(async move {
            run_polling_sync(
                &options_for_spawn,
                syncer_for_spawn.as_ref(),
                sleep_ctrl_for_spawn.sleep_fn(),
            ).await;
        });

        // these sleeps should wait for the maximum of the polling interval and the
        // sync cooldown
        for i in 0..30 {
            let cooldown_secs = calc_exp_backoff(
                options.sync_cooldown.base_secs,
                options.sync_cooldown.growth_factor,
                i + 1,
                options.sync_cooldown.max_secs
            );
            let expected_sleep_secs = max(
                options.poll_secs as u64,
                cooldown_secs as u64,
            );

            sleep_ctrl.await_sleep().await;
            let last_sleep = sleep_ctrl.get_last_sleep().unwrap();
            assert_eq!(expected_sleep_secs, last_sleep.as_secs());
            assert_eq!(syncer.num_sync_calls(), (i + 1) as usize);
            sleep_ctrl.release().await;
        }

        syncer.set_sync(|| Ok(()));

        // these sleeps should wait for the polling interval after recovery
        for i in 0..10 {
            sleep_ctrl.await_sleep().await;
            let last_sleep = sleep_ctrl.get_last_sleep().unwrap();
            assert_eq!(last_sleep.as_secs(), options.poll_secs as u64);
            assert_eq!(syncer.num_sync_calls(), i + 31);
            sleep_ctrl.release().await;
        }

    }
}

pub mod handle_mqtt_event {
    use super::*;

    #[tokio::test]
    async fn non_publish_event() {
        let event = Event::Incoming(Incoming::PingReq);
        let device_client = MockDeviceClient::default();
        let syncer = MockSyncer::default();
        handle_mqtt_event(
            &event,
            "device_id",
            &device_client,
            &syncer,
            &CooldownOptions::default(),
        ).await;

        assert_eq!(device_client.num_publish_device_sync_calls(), 0);
        assert_eq!(syncer.num_sync_calls(), 0);
    }

    #[tokio::test]
    async fn sync_request_unserializable() {
        let event = Event::Incoming(Incoming::Publish(Publish::new(
            "test".to_string(),
            QoS::AtLeastOnce,
            "test".to_string(),
        )));
        let device_client = MockDeviceClient::default();
        let syncer = MockSyncer::default();
        handle_mqtt_event(
            &event,
            "device_id",
            &device_client,
            &syncer,
            &CooldownOptions::default(),
        ).await;

        assert_eq!(device_client.num_publish_device_sync_calls(), 1);
        assert_eq!(syncer.num_sync_calls(), 1);
    }

    #[tokio::test]
    async fn sync_request_is_synced() {
        let payload = SyncDevice {
            is_synced: true,
        };
        let payload_bytes = serde_json::to_vec(&payload).unwrap();
        let event = Event::Incoming(Incoming::Publish(Publish::new(
            "test".to_string(),
            QoS::AtLeastOnce,
            payload_bytes,
        )));
        let device_client = MockDeviceClient::default();
        let syncer = MockSyncer::default();
        handle_mqtt_event(
            &event,
            "device_id",
            &device_client,
            &syncer,
            &CooldownOptions::default(),
        ).await;

        assert_eq!(device_client.num_publish_device_sync_calls(), 0);
        assert_eq!(syncer.num_sync_calls(), 0);
    }

    #[tokio::test]
    async fn sync_request_is_not_synced() {
        let payload = SyncDevice {
            is_synced: false,
        };
        let payload_bytes = serde_json::to_vec(&payload).unwrap();
        let event = Event::Incoming(Incoming::Publish(Publish::new(
            "test".to_string(),
            QoS::AtLeastOnce,
            payload_bytes,
        )));
        let device_client = MockDeviceClient::default();
        let syncer = MockSyncer::default();
        handle_mqtt_event(
            &event,
            "device_id",
            &device_client,
            &syncer,
            &CooldownOptions::default(),
        ).await;

        assert_eq!(device_client.num_publish_device_sync_calls(), 1);
        assert_eq!(syncer.num_sync_calls(), 1);
    }

    #[tokio::test]
    async fn sync_error() {
        let payload = SyncDevice {
            is_synced: false,
        };
        let payload_bytes = serde_json::to_vec(&payload).unwrap();
        let event = Event::Incoming(Incoming::Publish(Publish::new(
            "test".to_string(),
            QoS::AtLeastOnce,
            payload_bytes,
        )));
        let device_client = MockDeviceClient::default();
        let mut syncer = MockSyncer::default();
        syncer.set_sync(|| Err(SyncErr::MockErr(Box::new(SyncMockErr {
            is_network_connection_error: false,
        }))));
        handle_mqtt_event(
            &event,
            "device_id",
            &device_client,
            &syncer,
            &CooldownOptions::default(),
        ).await;

        assert_eq!(device_client.num_publish_device_sync_calls(), 0);
        assert_eq!(syncer.num_sync_calls(), 1);
    }
}

// pub mod handle_mqtt_error {
//     use super::*;

//     #[tokio::test]
//     async fn authentication_error_triggers_token_refresh() {
//         let token = Token {
//             token: "token".to_string(),
//             expires_at: Utc::now(),
//         };
//         let token_mngr = MockTokenManager::new(token);
//         let error = MQTTError::MockErr(Box::new(MockErr {
//             is_authentication_error: true,
//             is_network_connection_error: false,
//         }));
//         handle_mqtt_error(error, &token_mngr).await;
//         assert_eq!(token_mngr.num_refresh_token_calls(), 1);
//     }

//     #[tokio::test]
//     async fn other_errors_are_ignored() {
//         let token = Token {
//             token: "token".to_string(),
//             expires_at: Utc::now(),
//         };
//         let token_mngr = MockTokenManager::new(token);
//         let error = MQTTError::MockErr(Box::new(MockErr {
//             is_authentication_error: false,
//             is_network_connection_error: true,
//         }));
//         handle_mqtt_error(error, &token_mngr).await;
//         assert_eq!(token_mngr.num_refresh_token_calls(), 0);
//     }
// }