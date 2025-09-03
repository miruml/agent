// std crates
use std::path::PathBuf;
use std::sync::Arc;

// internal crates
use config_agent::app::state::AppState;
use config_agent::authn::token::Token;
use config_agent::deploy::fsm;
use config_agent::filesys::dir::Dir;
use config_agent::filesys::errors::FileSysErr;
use config_agent::http::client::HTTPClient;
use config_agent::logs::*;
use config_agent::models::{
    device,
    device::{Device, DeviceStatus},
};
use config_agent::server::errors::ServerErr;
use config_agent::storage::caches::CacheCapacities;
use config_agent::storage::device::DeviceFile;
use config_agent::storage::layout::StorageLayout;

use crate::authn::mock::MockTokenManager;
use crate::http::mock::MockDevicesClient;

// external crates
use chrono::Utc;

pub mod init {
    use super::*;

    #[tokio::test]
    async fn fail_missing_private_key_file() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);
        let result = AppState::init(
            Device::default().version,
            &layout,
            CacheCapacities::default(),
            Arc::new(HTTPClient::new("doesntmatter").await),
            fsm::Settings::default(),
        )
        .await;
        match result {
            Err(ServerErr::FileSysErr(e)) => {
                assert!(matches!(e.source, FileSysErr::PathDoesNotExistErr(_)));
            }
            Err(e) => {
                panic!("Expected FileSysErr not {e:?}");
            }
            Ok(_) => {
                panic!("expected error from initializing server state");
            }
        }
    }

    #[tokio::test]
    async fn fail_missing_device_id() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);
        // create a private key file
        let private_key_file = layout.auth_dir().private_key_file();
        private_key_file
            .write_string("test", false, false)
            .await
            .unwrap();

        let result = AppState::init(
            Device::default().version,
            &layout,
            CacheCapacities::default(),
            Arc::new(HTTPClient::new("doesntmatter").await),
            fsm::Settings::default(),
        )
        .await;
        assert!(matches!(result, Err(ServerErr::MissingDeviceIDErr(_))));
    }

    #[tokio::test]
    async fn success_missing_device_file_but_valid_token() {
        let begin_test = Utc::now().timestamp();
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);

        // create a private key file
        let private_key_file = layout.auth_dir().private_key_file();
        private_key_file
            .write_string("test", false, false)
            .await
            .unwrap();

        // create the token file with a token containing a device id
        let token_file = layout.auth_dir().token_file();
        let token = Token {
                token: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpYXQiOjE3NDU2MzgzMTUsInN1YiI6ImNsaV8xMjMiLCJpc3MiOiJtaXJ1IiwiYXVkIjoiY2xpZW50IiwiZXhwIjoxNzIxNTE3MDM0fQ.4ARFzYZSF_i9PjPZRJtH7HcmE_vv5tuZIpKkniua6BY".to_string(),
                expires_at: Utc::now(),
            };
        token_file.write_json(&token, false, false).await.unwrap();

        let (state, _) = AppState::init(
            Device::default().version,
            &layout,
            CacheCapacities::default(),
            Arc::new(HTTPClient::new("doesntmatter").await),
            fsm::Settings::default(),
        )
        .await
        .unwrap();

        // check last activity
        assert!(state.activity_tracker.last_touched() <= Utc::now().timestamp() as u64);
        assert!(state.activity_tracker.last_touched() >= begin_test as u64);

        // the device file should now exist with some reasonable defaults
        let device_file = layout.device_file();
        let expected_device = Device {
            id: "cli_123".to_string(),
            activated: true,
            status: DeviceStatus::Offline,
            ..Device::default()
        };
        let device = device_file.read_json::<Device>().await.unwrap();
        assert_eq!(device, expected_device);
    }

    #[tokio::test]
    async fn success_missing_token_file() {
        let begin_test = Utc::now().timestamp();
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);

        // create a private key file
        let private_key_file = layout.auth_dir().private_key_file();
        private_key_file
            .write_string("test", false, false)
            .await
            .unwrap();

        // create the device file
        let device_file = layout.device_file();
        let device = Device::default();
        device_file.write_json(&device, false, false).await.unwrap();

        let (state, _) = AppState::init(
            Device::default().version,
            &layout,
            CacheCapacities::default(),
            Arc::new(HTTPClient::new("doesntmatter").await),
            fsm::Settings::default(),
        )
        .await
        .unwrap();

        // the token file should now have the default token
        let token_file = layout.auth_dir().token_file();
        let token = token_file.read_json::<Token>().await.unwrap();
        assert_eq!(token.token, Token::default().token);

        // check last activity
        assert!(state.activity_tracker.last_touched() <= Utc::now().timestamp() as u64);
        assert!(state.activity_tracker.last_touched() >= begin_test as u64);
    }

    #[tokio::test]
    async fn success_set_device_to_offline_on_boot() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);

        // create a private key file
        let private_key_file = layout.auth_dir().private_key_file();
        private_key_file
            .write_string("test", false, false)
            .await
            .unwrap();

        // create the device file
        let device_file = layout.device_file();
        let device = Device {
            id: "dvc_123".to_string(),
            activated: true,
            status: DeviceStatus::Online,
            ..Device::default()
        };
        device_file.write_json(&device, false, false).await.unwrap();

        let _ = AppState::init(
            Device::default().version,
            &layout,
            CacheCapacities::default(),
            Arc::new(HTTPClient::new("doesntmatter").await),
            fsm::Settings::default(),
        )
        .await
        .unwrap();

        // the device file should now have the device set to offline
        let device_file = layout.device_file();
        let device = device_file.read_json::<Device>().await.unwrap();
        assert_eq!(device.status, DeviceStatus::Offline);
    }
}

pub mod update_agent_version {
    use super::*;

    #[tokio::test]
    async fn update_agent_version_same_version() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);


        let agent_version = Device::default().version;
        let device = Device {
            version: agent_version.clone(),
            ..Device::default()
        };

        let (device_file, _) = DeviceFile::spawn_with_default(
            64,
            layout.device_file(),
            device,
        )
        .await
        .unwrap();
        let http_client = MockDevicesClient::default();
        let token_mngr = MockTokenManager::new(Token::default());

        AppState::update_agent_version(
            &device_file,
            &http_client,
            &token_mngr,
            agent_version.clone(),
        )
        .await
        .unwrap();

        // check the device file has the same version
        let device = device_file.read().await.unwrap();
        assert_eq!(device.version, agent_version);

        // check the token manager has not been called
        assert_eq!(token_mngr.num_get_token_calls(), 0);

        // check the http client has been called
        assert_eq!(http_client.num_update_device_calls(), 0);
    }

    #[tokio::test]
    async fn update_agent_version_different_version() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);

        let old_agent_version = Device::default().version;
        let new_agent_version = "v1.0.1".to_string();
        let device = Device {
            version: old_agent_version.clone(),
            ..Device::default()
        };

        let (device_file, _) = DeviceFile::spawn_with_default(
            64,
            layout.device_file(),
            device,
        )
        .await
        .unwrap();
        let http_client = MockDevicesClient::default();
        let token_mngr = MockTokenManager::new(Token::default());

        AppState::update_agent_version(
            &device_file,
            &http_client,
            &token_mngr,
            new_agent_version.clone(),
        )
        .await
        .unwrap();

        // check the device file has the correct version
        let device = device_file.read().await.unwrap();
        assert_eq!(device.version, new_agent_version);

        // check the token manager has not been called
        assert_eq!(token_mngr.num_get_token_calls(), 1);

        // check the http client has been called
        assert_eq!(http_client.num_update_device_calls(), 1);
    }
}


pub mod shutdown {
    use super::*;

    #[tokio::test]
    async fn success_device_offline() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);

        // create a private key file
        let private_key_file = layout.auth_dir().private_key_file();
        private_key_file
            .write_string("test", false, false)
            .await
            .unwrap();

        // create the device file
        let device_file = layout.device_file();
        let device = Device::default();
        device_file.write_json(&device, false, false).await.unwrap();

        let (state, state_handle) = AppState::init(
            Device::default().version,
            &layout,
            CacheCapacities::default(),
            Arc::new(HTTPClient::new("doesntmatter").await),
            fsm::Settings::default(),
        )
        .await
        .unwrap();
        state.shutdown().await.unwrap();
        state_handle.await;
    }

    #[tokio::test]
    async fn success_device_online() {
        let _ = init(LogOptions {
            stdout: true,
            log_level: LogLevel::Info,
            log_dir: PathBuf::from("/tmp/miru"),
        });

        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);

        // create a private key file
        let private_key_file = layout.auth_dir().private_key_file();
        private_key_file
            .write_string("test", false, false)
            .await
            .unwrap();

        // create the device file
        let device_file = layout.device_file();
        let device = Device::default();
        device_file.write_json(&device, true, false).await.unwrap();

        let before_shutdown = Utc::now();
        let (state, state_handle) = AppState::init(
            Device::default().version,
            &layout,
            CacheCapacities::default(),
            Arc::new(HTTPClient::new("doesntmatter").await),
            fsm::Settings::default(),
        )
        .await
        .unwrap();

        // set the device to be online
        state
            .device_file
            .patch(device::Updates::connected())
            .await
            .unwrap();

        state.shutdown().await.unwrap();
        state_handle.await;

        // the device file should now have the device set to offline
        let device_file = layout.device_file();
        let device = device_file.read_json::<Device>().await.unwrap();
        assert_eq!(device.status, DeviceStatus::Offline);
        assert!(device.last_disconnected_at >= before_shutdown);
        assert!(device.last_disconnected_at <= Utc::now());
    }
}
