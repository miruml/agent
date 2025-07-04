// standard crates
use std::sync::Arc;

// internal crates
use crate::auth::token_mngr::spawn as spawn_token_manager;
use crate::http::mock::{MockAuthClient, MockConfigInstancesClient};
use config_agent::auth::{
    token_mngr::{TokenFile, TokenManager},
    token::Token,
};
use config_agent::crud::prelude::*;
use config_agent::deploy::fsm;
use config_agent::errors::*;
use config_agent::filesys::dir::Dir;
use config_agent::http::{
    client::HTTPClient,
    errors::{HTTPErr, MockErr},
};
use config_agent::models::config_instance::ActivityStatus;
use config_agent::storage::config_instances::{ConfigInstanceCache, ConfigInstanceDataCache};
use config_agent::sync::{
    errors::SyncErr,
    syncer::{SingleThreadSyncer, Syncer, Worker, SyncerArgs},
};
use config_agent::trace;

// external crates
use chrono::{DateTime, TimeDelta, Utc};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub async fn create_token_manager(
    dir: &Dir,
    http_client: Arc<MockAuthClient>,
) -> (TokenManager, JoinHandle<()>) {
    let token_file = TokenFile::new_with_default(dir.file("token.json"), Token::default())
        .await
        .unwrap();
    let private_key_file = dir.file("private_key.pem");
    private_key_file
        .write_string("private_key", true, true)
        .await
        .unwrap();

    spawn_token_manager(
        32,
        "device_id".to_string(),
        http_client.clone(),
        token_file,
        private_key_file,
    )
    .unwrap()
}

pub struct TestSyncerArgs {
    pub device_id: String,
    pub http_client: Arc<MockConfigInstancesClient>,
    pub token_mngr: Arc<TokenManager>,
    pub cfg_inst_cache: Arc<ConfigInstanceCache>,
    pub cfg_inst_data_cache: Arc<ConfigInstanceDataCache>,
    pub deployment_dir: Dir,
    pub fsm_settings: fsm::Settings,
}

pub fn spawn(
    buffer_size: usize,
    args: TestSyncerArgs,
) -> Result<(Syncer, JoinHandle<()>), SyncErr> {
    let (sender, receiver) = mpsc::channel(buffer_size);
    let worker = Worker::new(
        SingleThreadSyncer::new(
            args.device_id,
            args.http_client,
            args.token_mngr,
            args.cfg_inst_cache,
            args.cfg_inst_data_cache,
            args.deployment_dir,
            args.fsm_settings,
        ),
        receiver,
    );
    let worker_handle = tokio::spawn(worker.run());
    Ok((Syncer::new(sender), worker_handle))
}

pub mod shutdown {
    use super::*;

    #[tokio::test]
    async fn shutdown() {
        let dir = Dir::create_temp_dir("spawn").await.unwrap();
        let auth_client = Arc::new(MockAuthClient::default());
        let (token_mngr, _) = create_token_manager(&dir, auth_client.clone()).await;

        // create the caches
        let (cfg_inst_cache, _) = ConfigInstanceCache::spawn(16, dir.file("cfg_inst_cache.json"), 1000)
            .await
            .unwrap();
        let (cfg_inst_data_cache, _) = ConfigInstanceDataCache::spawn(16, dir.subdir("cfg_inst_data_cache"), 1000)
            .await
            .unwrap();

        let http_client = Arc::new(HTTPClient::new("doesntmatter").await);
        let (syncer, worker_handler) = Syncer::spawn(
            32,
            SyncerArgs {
                device_id: "device_id".to_string(),
                http_client: http_client.clone(),
                token_mngr: Arc::new(token_mngr),
                cfg_inst_cache: Arc::new(cfg_inst_cache),
                cfg_inst_data_cache: Arc::new(cfg_inst_data_cache),
                deployment_dir: dir,
                fsm_settings: fsm::Settings::default(),
            },
        )
        .unwrap();

        syncer.shutdown().await.unwrap();
        worker_handler.await.unwrap();
    }
}

pub mod sync {
    use super::*;

    #[tokio::test]
    async fn network_error() {
        let dir = Dir::create_temp_dir("spawn").await.unwrap();
        let auth_client = Arc::new(MockAuthClient::default());
        let (token_mngr, _) = create_token_manager(&dir, auth_client.clone()).await;

        let mut http_client = MockConfigInstancesClient::default();
        http_client.set_list_all_config_instances(|| {
            Err(HTTPErr::MockErr(Box::new(MockErr {
                is_network_connection_error: true,
                trace: trace!(),
            })))
        });
        http_client.set_update_config_instance(|| {
            Err(HTTPErr::MockErr(Box::new(MockErr {
                is_network_connection_error: true,
                trace: trace!(),
            })))
        });

        // create the caches
        let (cfg_inst_cache, _) = ConfigInstanceCache::spawn(16, dir.file("cfg_inst_cache.json"), 1000)
            .await
            .unwrap();
        let (cfg_inst_data_cache, _) =
            ConfigInstanceDataCache::spawn(16, dir.subdir("cfg_inst_data_cache"), 1000)
                .await
                .unwrap();

        let (syncer, _) = spawn(
            32,
            TestSyncerArgs {
                device_id: "device_id".to_string(),
                http_client: Arc::new(http_client),
                token_mngr: Arc::new(token_mngr),
                cfg_inst_cache: Arc::new(cfg_inst_cache),
                cfg_inst_data_cache: Arc::new(cfg_inst_data_cache),
                deployment_dir: dir.clone(),
                fsm_settings: fsm::Settings::default(),
            },
        )
        .unwrap();

        let error = syncer.sync(TimeDelta::seconds(1)).await.unwrap_err();
        assert!(error.is_network_connection_error());
    }

    #[tokio::test]
    async fn pull_deploy_and_push() {
        let dir = Dir::create_temp_dir("spawn").await.unwrap();
        let auth_client = Arc::new(MockAuthClient::default());
        let (token_mngr, _) = create_token_manager(&dir, auth_client.clone()).await;

        // define the new instance
        let id = "new_instance".to_string();
        let new_instance_data = serde_json::json!({"id": id});
        let new_instance = openapi_client::models::BackendConfigInstance {
            id: id.clone(),
            target_status: openapi_client::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_DEPLOYED,
            instance: Some(new_instance_data.clone()),
            ..Default::default()
        };
        let mut http_client = MockConfigInstancesClient::default();
        let new_instance_cloned = new_instance.clone();
        http_client.set_list_all_config_instances(move || Ok(vec![new_instance_cloned.clone()]));

        // create the caches
        let (cfg_inst_cache, _) = ConfigInstanceCache::spawn(16, dir.file("cfg_inst_cache.json"), 1000)
            .await
            .unwrap();
        let (cfg_inst_data_cache, _) =
            ConfigInstanceDataCache::spawn(16, dir.subdir("cfg_inst_data_cache"), 1000)
                .await
                .unwrap();
        let cfg_inst_cache = Arc::new(cfg_inst_cache);
        let cfg_inst_data_cache = Arc::new(cfg_inst_data_cache);

        let (syncer, _) = spawn(
            32,
            TestSyncerArgs {
                device_id: "device_id".to_string(),
                http_client: Arc::new(http_client),
                token_mngr: Arc::new(token_mngr),
                cfg_inst_cache: cfg_inst_cache.clone(),
                cfg_inst_data_cache: cfg_inst_data_cache.clone(),
                deployment_dir: dir.clone(),
                fsm_settings: fsm::Settings::default(),
            },
        )
        .unwrap();

        syncer.sync(TimeDelta::seconds(1)).await.unwrap();

        // check the metadata cache has the new instance
        let cache_cfg_inst = cfg_inst_cache.read(id.clone()).await.unwrap();
        assert_eq!(cache_cfg_inst.activity_status, ActivityStatus::Deployed);

        // check the data cache has the new instance data
        let cache_cfg_inst_data = cfg_inst_data_cache.read(id.clone()).await.unwrap();
        assert_eq!(cache_cfg_inst_data, new_instance_data);

        // check that the metadata cache isn't dirty
        let unsynced_entries = cfg_inst_cache.get_dirty_entries().await.unwrap();
        assert_eq!(unsynced_entries.len(), 0);
    }

    #[tokio::test]
    async fn in_cooldown() {
        let dir = Dir::create_temp_dir("spawn").await.unwrap();
        let auth_client = Arc::new(MockAuthClient::default());
        let (token_mngr, _) = create_token_manager(&dir, auth_client.clone()).await;

        // create the caches
        let (cfg_inst_cache, _) = ConfigInstanceCache::spawn(16, dir.file("cfg_inst_cache.json"), 1000)
            .await
            .unwrap();
        let (cfg_inst_data_cache, _) =
            ConfigInstanceDataCache::spawn(16, dir.subdir("cfg_inst_data_cache"), 1000)
                .await
                .unwrap();

        let (syncer, _) = spawn(
            32,
            TestSyncerArgs {
                device_id: "device_id".to_string(),
                http_client: Arc::new(MockConfigInstancesClient::default()),
                token_mngr: Arc::new(token_mngr),
                cfg_inst_cache: Arc::new(cfg_inst_cache),
                cfg_inst_data_cache: Arc::new(cfg_inst_data_cache),
                deployment_dir: dir.clone(),
                fsm_settings: fsm::Settings::default(),
            },
        )
        .unwrap();

        let before = Utc::now();
        syncer.sync(TimeDelta::seconds(1)).await.unwrap();
        let after = Utc::now();

        assert!(before < syncer.get_last_synced_at().await.unwrap());
        assert!(syncer.get_last_synced_at().await.unwrap() < after);
        let prev_synced_at = syncer.get_last_synced_at().await.unwrap();

        // syncing again should not change the last synced at
        syncer.sync(TimeDelta::seconds(1)).await.unwrap();
        assert_eq!(syncer.get_last_synced_at().await.unwrap(), prev_synced_at);
    }
}

pub mod get_last_synced_at {
    use super::*;

    #[tokio::test]
    async fn get_last_synced_at() {
        let dir = Dir::create_temp_dir("spawn").await.unwrap();
        let auth_client = Arc::new(MockAuthClient::default());
        let (token_mngr, _) = create_token_manager(&dir, auth_client.clone()).await;

        // create the caches
        let (cfg_inst_cache, _) = ConfigInstanceCache::spawn(16, dir.file("cfg_inst_cache.json"), 1000)
            .await
            .unwrap();
        let (cfg_inst_data_cache, _) =
            ConfigInstanceDataCache::spawn(16, dir.subdir("cfg_inst_data_cache"), 1000)
                .await
                .unwrap();

        let (syncer, _) = spawn(
            32,
            TestSyncerArgs {
                device_id: "device_id".to_string(),
                http_client: Arc::new(MockConfigInstancesClient::default()),
                token_mngr: Arc::new(token_mngr),
                cfg_inst_cache: Arc::new(cfg_inst_cache),
                cfg_inst_data_cache: Arc::new(cfg_inst_data_cache),
                deployment_dir: dir.clone(),
                fsm_settings: fsm::Settings::default(),
            },
        )
        .unwrap();

        assert_eq!(
            syncer.get_last_synced_at().await.unwrap(),
            DateTime::<Utc>::UNIX_EPOCH
        );

        let before = Utc::now();
        syncer.sync(TimeDelta::seconds(1)).await.unwrap();
        let after = Utc::now();

        assert!(before < syncer.get_last_synced_at().await.unwrap());
        assert!(syncer.get_last_synced_at().await.unwrap() < after);
    }
}
