// standard crates
use std::sync::Arc;

// internal crates
use config_agent::authn::token::Token;
use config_agent::filesys::dir::Dir;
use config_agent::models::device::{Device, DeviceStatus};
use config_agent::mqtt::{
    client::{MQTTClient, Options},
    device::SyncDevice,
    errors::*,
};
use config_agent::storage::{device::DeviceFile, layout::StorageLayout};
use config_agent::sync::{
    errors::{MockErr as SyncMockErr, SyncErr},
    syncer::{CooldownEnd, SyncEvent, SyncFailure, SyncState},
};
use config_agent::workers::poller::{
    handle_mqtt_error, handle_mqtt_event, handle_syncer_event, run_polling_sync_worker,
    BackendSyncWorkerOptions, MqttState,
};

use crate::authn::mock::MockTokenManager;
use crate::mock::SleepController;
use crate::mqtt::mock::MockDeviceClient;
use crate::sync::mock::MockSyncer;

// external crates
use chrono::{TimeDelta, Utc};
use rumqttc::{ConnAck, ConnectReturnCode, Event, Incoming, Publish, QoS};

pub mod run_polling_sync {
    use super::*;

    #[tokio::test]
    async fn syncer_not_in_cooldown() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);

        let (device_file, _) =
            DeviceFile::spawn_with_default(64, layout.device_file(), Device::default())
                .await
                .unwrap();

        let options = BackendSyncWorkerOptions::default();
        let syncer = Arc::new(MockSyncer::default());
        let sleep_ctrl = Arc::new(SleepController::new());

        let syncer_for_spawn = syncer.clone();
        let sleep_ctrl_for_spawn = sleep_ctrl.clone();
        let _handle = tokio::spawn(async move {
            run_polling_sync_worker(
                options.poll_interval_secs,
                syncer_for_spawn.as_ref(),
                &device_file,
                sleep_ctrl_for_spawn.sleep_fn(),
            )
            .await;
        });

        let secs_since_last_sync = 30;
        let state = SyncState {
            last_attempted_sync_at: Utc::now() - TimeDelta::seconds(secs_since_last_sync),
            last_synced_at: Utc::now(),
            cooldown_ends_at: Utc::now(),
            err_streak: 0,
        };
        syncer.set_state(state);

        // these sleeps should wait for the polling interval since it exceeds the syncer
        let expected_sleep_secs = options.poll_interval_secs - secs_since_last_sync;
        // cooldown
        for i in 0..10 {
            sleep_ctrl.await_sleep().await;
            let last_sleep = sleep_ctrl.get_last_attempted_sleep().unwrap();
            assert!(last_sleep.as_secs() <= expected_sleep_secs as u64);
            assert!(last_sleep.as_secs() >= expected_sleep_secs as u64 - 1);
            assert_eq!(syncer.num_sync_calls(), i + 1);
            sleep_ctrl.release().await;
        }

        // these sleeps should still wait for the polling interval starting from the
        // last sync attempt since errors are logged & ignored
        syncer.set_sync(|| {
            Err(SyncErr::MockErr(Box::new(SyncMockErr {
                is_network_connection_error: true,
            })))
        });
        for i in 0..10 {
            sleep_ctrl.await_sleep().await;
            let last_sleep = sleep_ctrl.get_last_attempted_sleep().unwrap();
            assert!(last_sleep.as_secs() <= expected_sleep_secs as u64);
            assert!(last_sleep.as_secs() >= expected_sleep_secs as u64 - 1);
            assert_eq!(syncer.num_sync_calls(), i + 11);
            sleep_ctrl.release().await;
        }

        syncer.set_sync(|| {
            Err(SyncErr::MockErr(Box::new(SyncMockErr {
                is_network_connection_error: false,
            })))
        });
        for i in 0..10 {
            sleep_ctrl.await_sleep().await;
            let last_sleep = sleep_ctrl.get_last_attempted_sleep().unwrap();
            assert!(last_sleep.as_secs() <= expected_sleep_secs as u64);
            assert!(last_sleep.as_secs() >= expected_sleep_secs as u64 - 1);
            assert_eq!(syncer.num_sync_calls(), i + 21);
            sleep_ctrl.release().await;
        }
    }

    #[tokio::test]
    async fn syncer_in_cooldown() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);

        let (device_file, _) =
            DeviceFile::spawn_with_default(64, layout.device_file(), Device::default())
                .await
                .unwrap();

        let options = BackendSyncWorkerOptions {
            poll_interval_secs: 30,
            ..Default::default()
        };
        let syncer = Arc::new(MockSyncer::default());
        let sleep_ctrl = Arc::new(SleepController::new());

        let syncer_for_spawn = syncer.clone();
        let sleep_ctrl_for_spawn = sleep_ctrl.clone();
        let _handle = tokio::spawn(async move {
            run_polling_sync_worker(
                options.poll_interval_secs,
                syncer_for_spawn.as_ref(),
                &device_file,
                sleep_ctrl_for_spawn.sleep_fn(),
            )
            .await;
        });

        let secs_until_cooldown_ends = 120;
        let state = SyncState {
            last_attempted_sync_at: Utc::now(),
            last_synced_at: Utc::now(),
            cooldown_ends_at: Utc::now() + TimeDelta::seconds(secs_until_cooldown_ends),
            err_streak: 0,
        };
        syncer.set_state(state);

        // these sleeps should wait for the syncer cooldown to end since it's greater
        // than the polling interval
        for _ in 0..10 {
            sleep_ctrl.await_sleep().await;
            let last_sleep = sleep_ctrl.get_last_attempted_sleep().unwrap();
            assert!(last_sleep.as_secs() <= secs_until_cooldown_ends as u64);
            assert!(last_sleep.as_secs() >= secs_until_cooldown_ends as u64 - 1);
            assert_eq!(syncer.num_sync_calls(), 0); // syncer in cooldown
            sleep_ctrl.release().await;
        }

        // these sleeps should still wait for the syncer cooldown to end since errors
        // are logged & ignored
        syncer.set_sync(|| {
            Err(SyncErr::MockErr(Box::new(SyncMockErr {
                is_network_connection_error: true,
            })))
        });
        for _ in 0..10 {
            sleep_ctrl.await_sleep().await;
            let last_sleep = sleep_ctrl.get_last_attempted_sleep().unwrap();
            assert!(last_sleep.as_secs() <= secs_until_cooldown_ends as u64);
            assert!(last_sleep.as_secs() >= secs_until_cooldown_ends as u64 - 1);
            assert_eq!(syncer.num_sync_calls(), 0); // syncer in cooldown
            sleep_ctrl.release().await;
        }

        syncer.set_sync(|| {
            Err(SyncErr::MockErr(Box::new(SyncMockErr {
                is_network_connection_error: false,
            })))
        });
        for _ in 0..10 {
            sleep_ctrl.await_sleep().await;
            let last_sleep = sleep_ctrl.get_last_completed_sleep().unwrap();
            assert!(last_sleep.as_secs() <= secs_until_cooldown_ends as u64);
            assert!(last_sleep.as_secs() >= secs_until_cooldown_ends as u64 - 1);
            assert_eq!(syncer.num_sync_calls(), 0); // syncer in cooldown
            sleep_ctrl.release().await;
        }
    }

    #[tokio::test]
    async fn ignored_syncer_events() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);

        let (device_file, _) =
            DeviceFile::spawn_with_default(64, layout.device_file(), Device::default())
                .await
                .unwrap();

        let options = BackendSyncWorkerOptions::default();
        let syncer = Arc::new(MockSyncer::default());
        let sleep_ctrl = Arc::new(SleepController::new());

        let syncer_for_spawn = syncer.clone();
        let sleep_ctrl_for_spawn = sleep_ctrl.clone();
        let _handle = tokio::spawn(async move {
            run_polling_sync_worker(
                options.poll_interval_secs,
                syncer_for_spawn.as_ref(),
                &device_file,
                sleep_ctrl_for_spawn.sleep_fn(),
            )
            .await;
        });

        let secs_since_last_sync = 30;
        let state = SyncState {
            last_attempted_sync_at: Utc::now() - TimeDelta::seconds(secs_since_last_sync),
            last_synced_at: Utc::now(),
            cooldown_ends_at: Utc::now(),
            err_streak: 0,
        };
        syncer.set_state(state);

        let syncer_tx = syncer.get_transmitter();

        let expected_sleep_secs = options.poll_interval_secs - secs_since_last_sync;
        let expected_num_sync_calls = 1; // only the first sync occurs
        for event in [
            SyncEvent::SyncSuccess,
            SyncEvent::SyncFailed(SyncFailure {
                is_network_connection_error: true,
            }),
            SyncEvent::CooldownEnd(CooldownEnd::FromSyncSuccess),
        ] {
            for _ in 0..10 {
                sleep_ctrl.await_sleep().await;
                let last_sleep = sleep_ctrl.get_last_attempted_sleep().unwrap();
                assert!(last_sleep.as_secs() <= expected_sleep_secs as u64);
                assert!(last_sleep.as_secs() >= expected_sleep_secs as u64 - 1);
                assert_eq!(syncer.num_sync_calls(), expected_num_sync_calls);
                syncer_tx.send(event.clone()).unwrap();
            }
        }
    }

    #[tokio::test]
    async fn syncer_cooldown_end_from_sync_failure_event() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);

        let (device_file, _) =
            DeviceFile::spawn_with_default(64, layout.device_file(), Device::default())
                .await
                .unwrap();

        let options = BackendSyncWorkerOptions::default();
        let syncer = Arc::new(MockSyncer::default());
        let sleep_ctrl = Arc::new(SleepController::new());

        let syncer_for_spawn = syncer.clone();
        let sleep_ctrl_for_spawn = sleep_ctrl.clone();
        let _handle = tokio::spawn(async move {
            run_polling_sync_worker(
                options.poll_interval_secs,
                syncer_for_spawn.as_ref(),
                &device_file,
                sleep_ctrl_for_spawn.sleep_fn(),
            )
            .await;
        });

        let secs_since_last_sync = 45;
        let state = SyncState {
            last_attempted_sync_at: Utc::now() - TimeDelta::seconds(secs_since_last_sync),
            last_synced_at: Utc::now(),
            cooldown_ends_at: Utc::now() - TimeDelta::seconds(10),
            err_streak: 0,
        };
        syncer.set_state(state);

        let syncer_tx = syncer.get_transmitter();

        let expected_sleep_secs = options.poll_interval_secs - secs_since_last_sync;
        let mut expected_num_sync_calls = 0; // only the first sync occurs
        for _ in 0..10 {
            expected_num_sync_calls += 1;
            sleep_ctrl.await_sleep().await;
            let last_attempted_sleep = sleep_ctrl.get_last_attempted_sleep().unwrap();
            assert!(last_attempted_sleep.as_secs() <= expected_sleep_secs as u64);
            assert!(last_attempted_sleep.as_secs() >= expected_sleep_secs as u64 - 1);
            assert_eq!(syncer.num_sync_calls(), expected_num_sync_calls);
            syncer_tx
                .send(SyncEvent::CooldownEnd(CooldownEnd::FromSyncFailure))
                .unwrap();
        }
    }
}

pub mod handle_syncer_event {
    use super::*;

    #[tokio::test]
    async fn sync_success_publishes_sync_to_backend() {
        let event = SyncEvent::SyncSuccess;
        let mqtt_client = MockDeviceClient::default();
        handle_syncer_event(&event, "device_id", &mqtt_client).await;
        assert_eq!(mqtt_client.num_publish_device_sync_calls(), 1);
    }

    #[tokio::test]
    async fn ignored_syncer_events() {
        for event in [
            SyncEvent::SyncFailed(SyncFailure {
                is_network_connection_error: true,
            }),
            SyncEvent::CooldownEnd(CooldownEnd::FromSyncSuccess),
            SyncEvent::CooldownEnd(CooldownEnd::FromSyncFailure),
        ] {
            let mqtt_client = MockDeviceClient::default();
            handle_syncer_event(&event, "device_id", &mqtt_client).await;
            assert_eq!(mqtt_client.num_publish_device_sync_calls(), 0);
        }
    }
}

pub mod handle_mqtt_event {
    use super::*;

    #[tokio::test]
    async fn unsuccessful_connack_event_is_ignored() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);

        let (device_file, _) =
            DeviceFile::spawn_with_default(64, layout.device_file(), Device::default())
                .await
                .unwrap();

        let event = Event::Incoming(Incoming::ConnAck(ConnAck {
            code: ConnectReturnCode::RefusedProtocolVersion,
            session_present: false,
        }));
        let syncer = MockSyncer::default();
        let err_streak = handle_mqtt_event(&event, &syncer, &device_file).await;
        assert_eq!(err_streak, 0);

        assert_eq!(syncer.num_sync_calls(), 0);
    }

    #[tokio::test]
    async fn successful_connack_event() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);

        let (device_file, _) = DeviceFile::spawn_with_default(
            64,
            layout.device_file(),
            Device {
                status: DeviceStatus::Offline,
                last_connected_at: Utc::now(),
                ..Device::default()
            },
        )
        .await
        .unwrap();

        let event = Event::Incoming(Incoming::ConnAck(ConnAck {
            code: ConnectReturnCode::Success,
            session_present: false,
        }));
        let syncer = MockSyncer::default();
        let before_event = Utc::now();
        let err_streak = handle_mqtt_event(&event, &syncer, &device_file).await;
        assert_eq!(err_streak, 0);

        let device = device_file.read().await.unwrap();
        assert_eq!(device.status, DeviceStatus::Online);
        assert!(device.last_connected_at >= before_event);
        assert!(device.last_connected_at <= Utc::now());
    }

    #[tokio::test]
    async fn disconnect_event() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);

        let (device_file, _) = DeviceFile::spawn_with_default(
            64,
            layout.device_file(),
            Device {
                status: DeviceStatus::Online,
                last_disconnected_at: Utc::now(),
                ..Device::default()
            },
        )
        .await
        .unwrap();

        let event = Event::Incoming(Incoming::Disconnect);
        let syncer = MockSyncer::default();
        let before_event = Utc::now();
        let err_streak = handle_mqtt_event(&event, &syncer, &device_file).await;
        assert_eq!(err_streak, 0);

        let device = device_file.read().await.unwrap();
        assert_eq!(device.status, DeviceStatus::Offline);
        assert!(device.last_disconnected_at >= before_event);
        assert!(device.last_disconnected_at <= Utc::now());
    }

    #[tokio::test]
    async fn sync_request_unserializable() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);

        let (device_file, _) =
            DeviceFile::spawn_with_default(64, layout.device_file(), Device::default())
                .await
                .unwrap();

        let event = Event::Incoming(Incoming::Publish(Publish::new(
            "test".to_string(),
            QoS::AtLeastOnce,
            "test".to_string(),
        )));
        let syncer = MockSyncer::default();
        let err_streak = handle_mqtt_event(&event, &syncer, &device_file).await;
        assert_eq!(err_streak, 0);

        assert_eq!(syncer.num_sync_calls(), 1);
    }

    #[tokio::test]
    async fn sync_request_is_synced() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);

        let (device_file, _) =
            DeviceFile::spawn_with_default(64, layout.device_file(), Device::default())
                .await
                .unwrap();

        let payload = SyncDevice { is_synced: true };
        let payload_bytes = serde_json::to_vec(&payload).unwrap();
        let event = Event::Incoming(Incoming::Publish(Publish::new(
            "test".to_string(),
            QoS::AtLeastOnce,
            payload_bytes,
        )));
        let syncer = MockSyncer::default();
        let err_streak = handle_mqtt_event(&event, &syncer, &device_file).await;
        assert_eq!(err_streak, 0);

        assert_eq!(syncer.num_sync_calls(), 0);
    }

    #[tokio::test]
    async fn sync_request_is_not_synced() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);

        let (device_file, _) =
            DeviceFile::spawn_with_default(64, layout.device_file(), Device::default())
                .await
                .unwrap();

        let payload = SyncDevice { is_synced: false };
        let payload_bytes = serde_json::to_vec(&payload).unwrap();
        let event = Event::Incoming(Incoming::Publish(Publish::new(
            "test".to_string(),
            QoS::AtLeastOnce,
            payload_bytes,
        )));
        let syncer = MockSyncer::default();
        let err_streak = handle_mqtt_event(&event, &syncer, &device_file).await;
        assert_eq!(err_streak, 0);

        assert_eq!(syncer.num_sync_calls(), 1);
    }

    #[tokio::test]
    async fn sync_error() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);

        let (device_file, _) =
            DeviceFile::spawn_with_default(64, layout.device_file(), Device::default())
                .await
                .unwrap();

        let payload = SyncDevice { is_synced: false };
        let payload_bytes = serde_json::to_vec(&payload).unwrap();
        let event = Event::Incoming(Incoming::Publish(Publish::new(
            "test".to_string(),
            QoS::AtLeastOnce,
            payload_bytes,
        )));
        let syncer = MockSyncer::default();
        syncer.set_sync(|| {
            Err(SyncErr::MockErr(Box::new(SyncMockErr {
                is_network_connection_error: false,
            })))
        });
        let err_streak = handle_mqtt_event(&event, &syncer, &device_file).await;
        assert_eq!(err_streak, 0);

        assert_eq!(syncer.num_sync_calls(), 1);
    }
}

pub mod handle_mqtt_error {
    use super::*;

    #[tokio::test]
    async fn authentication_error_triggers_token_refresh() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);

        let device = Device {
            id: "device_id".to_string(),
            session_id: "device_session_id".to_string(),
            status: DeviceStatus::Offline,
            ..Device::default()
        };
        let (device_file, _) =
            DeviceFile::spawn_with_default(64, layout.device_file(), device.clone())
                .await
                .unwrap();

        let token = Token {
            token: "token".to_string(),
            expires_at: Utc::now(),
        };
        let token_mngr = MockTokenManager::new(token);
        let error = MQTTError::MockErr(Box::new(MockErr {
            is_authentication_error: true,
            is_network_connection_error: false,
        }));

        let options = Options::default();
        let (mqtt_client, eventloop) = MQTTClient::new(&options).await;
        let created_at = mqtt_client.created_at;

        let before_patch = Utc::now();
        let mqtt_state = MqttState {
            mqtt_client,
            eventloop,
            err_streak: 2,
        };
        let mqtt_state = handle_mqtt_error(
            mqtt_state,
            error,
            &device,
            &token_mngr,
            &options.connect_address,
            &device_file,
        )
        .await;
        assert_eq!(token_mngr.num_refresh_token_calls(), 1);

        // should increment the error streak
        assert_eq!(mqtt_state.err_streak, 3);

        // should reinitialize the mqtt client
        assert_ne!(mqtt_state.mqtt_client.created_at, created_at);

        // shouldn't update the last disconnected at time since it was already offline
        let device = device_file.read().await.unwrap();
        assert_eq!(device.status, DeviceStatus::Offline);
        assert!(device.last_disconnected_at <= before_patch);
    }

    #[tokio::test]
    async fn other_errors_are_ignored() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);

        let device = Device {
            id: "device_id".to_string(),
            session_id: "device_session_id".to_string(),
            status: DeviceStatus::Online,
            ..Device::default()
        };
        let (device_file, _) =
            DeviceFile::spawn_with_default(64, layout.device_file(), device.clone())
                .await
                .unwrap();

        let token = Token {
            token: "token".to_string(),
            expires_at: Utc::now(),
        };
        let token_mngr = MockTokenManager::new(token);
        let error = MQTTError::MockErr(Box::new(MockErr {
            is_authentication_error: false,
            is_network_connection_error: true,
        }));

        let options = Options::default();
        let (mqtt_client, eventloop) = MQTTClient::new(&options).await;
        let created_at = mqtt_client.created_at;

        let before_patch = Utc::now();
        let mqtt_state = MqttState {
            mqtt_client,
            eventloop,
            err_streak: 1,
        };
        let mqtt_state = handle_mqtt_error(
            mqtt_state,
            error,
            &device,
            &token_mngr,
            &options.connect_address,
            &device_file,
        )
        .await;
        assert_eq!(token_mngr.num_refresh_token_calls(), 0);

        // should not increment the error streak
        assert_eq!(mqtt_state.err_streak, 1);

        // should not reinitialize the mqtt client
        assert_eq!(mqtt_state.mqtt_client.created_at, created_at);

        // should patch the device file to disconnected since it was online
        let device = device_file.read().await.unwrap();
        assert_eq!(device.status, DeviceStatus::Offline);
        assert!(device.last_disconnected_at >= before_patch);
        assert!(device.last_disconnected_at <= Utc::now());
    }
}
