// standard crates
use std::sync::{Arc, Mutex};

// internal crates
use config_agent::auth::{
    token::Token,
    token_mngr::{TokenFile, TokenManager},
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
use config_agent::storage::config_instances::{ConfigInstanceCache, ConfigInstanceContentCache};
use config_agent::sync::{
    errors::SyncErr,
    syncer::{
        CooldownEnd, SingleThreadSyncer, SyncEvent, SyncFailure, SyncState, Syncer, SyncerArgs,
        SyncerExt, Worker,
    },
};
use config_agent::utils::{calc_exp_backoff, CooldownOptions};

use crate::auth::token_mngr::spawn as spawn_token_manager;
use crate::http::mock::{MockAuthClient, MockConfigInstancesClient};

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

pub fn spawn(
    buffer_size: usize,
    args: SyncerArgs<MockConfigInstancesClient, TokenManager>,
) -> Result<(Syncer, JoinHandle<()>), SyncErr> {
    let (sender, receiver) = mpsc::channel(buffer_size);
    let worker = Worker::new(SingleThreadSyncer::new(args), receiver);
    let worker_handle = tokio::spawn(worker.run());
    Ok((Syncer::new(sender), worker_handle))
}

pub mod sync_state {
    use super::*;

    #[tokio::test]
    async fn is_in_cooldown() {
        // not in cooldown
        let state = SyncState {
            last_sync_attempted_at: Utc::now(),
            last_successful_sync_at: Utc::now(),
            cooldown_ends_at: Utc::now() + TimeDelta::seconds(10),
            err_streak: 0,
        };
        assert!(state.is_in_cooldown());

        // in cooldown
        let state = SyncState {
            last_sync_attempted_at: Utc::now(),
            last_successful_sync_at: Utc::now(),
            cooldown_ends_at: Utc::now() - TimeDelta::seconds(10),
            err_streak: 0,
        };
        assert!(!state.is_in_cooldown());
    }
}

pub mod shutdown {
    use super::*;

    #[tokio::test]
    async fn shutdown() {
        let dir = Dir::create_temp_dir("spawn").await.unwrap();
        let auth_client = Arc::new(MockAuthClient::default());
        let (token_mngr, _) = create_token_manager(&dir, auth_client.clone()).await;

        // create the caches
        let (cfg_inst_cache, _) =
            ConfigInstanceCache::spawn(16, dir.file("cfg_inst_cache.json"), 1000)
                .await
                .unwrap();
        let (cfg_inst_content_cache, _) =
            ConfigInstanceContentCache::spawn(16, dir.subdir("cfg_inst_content_cache"), 1000)
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
                cfg_inst_content_cache: Arc::new(cfg_inst_content_cache),
                deployment_dir: dir,
                fsm_settings: fsm::Settings::default(),
                cooldown_options: CooldownOptions::default(),
            },
        )
        .unwrap();

        syncer.shutdown().await.unwrap();
        worker_handler.await.unwrap();
    }
}

pub mod subscribe {
    use super::*;

    #[tokio::test]
    async fn sync_success() {
        let dir = Dir::create_temp_dir("spawn").await.unwrap();
        let auth_client = Arc::new(MockAuthClient::default());
        let (token_mngr, _) = create_token_manager(&dir, auth_client.clone()).await;
        let http_client = Arc::new(MockConfigInstancesClient::default());

        // create the caches
        let (cfg_inst_cache, _) =
            ConfigInstanceCache::spawn(16, dir.file("cfg_inst_cache.json"), 1000)
                .await
                .unwrap();
        let (cfg_inst_content_cache, _) =
            ConfigInstanceContentCache::spawn(16, dir.subdir("cfg_inst_content_cache"), 1000)
                .await
                .unwrap();

        let cooldown_options = CooldownOptions {
            base_secs: 1,
            ..CooldownOptions::default()
        };
        let (syncer, _) = spawn(
            32,
            SyncerArgs {
                device_id: "device_id".to_string(),
                http_client: http_client.clone(),
                token_mngr: Arc::new(token_mngr),
                cfg_inst_cache: Arc::new(cfg_inst_cache),
                cfg_inst_content_cache: Arc::new(cfg_inst_content_cache),
                deployment_dir: dir.clone(),
                fsm_settings: fsm::Settings::default(),
                cooldown_options,
            },
        )
        .unwrap();

        let mut subscriber = syncer.subscribe().await.unwrap();
        let events = Arc::new(Mutex::new(vec![]));

        let mut subscriber_for_spawn = subscriber.clone();
        let events_for_spawn = events.clone();
        let handle = tokio::spawn(async move {
            // expect two events: 1. not synced and then 2. cooldown ended
            for _ in 0..2 {
                subscriber_for_spawn.changed().await.unwrap();
                events_for_spawn
                    .lock()
                    .unwrap()
                    .push(subscriber_for_spawn.borrow().clone());
            }
        });

        syncer.sync().await.unwrap();
        // Wait for the cooldown to end
        loop {
            subscriber.changed().await.unwrap();
            let event = subscriber.borrow().clone();
            if matches!(event, SyncEvent::CooldownEnd(CooldownEnd::FromSyncSuccess)) {
                break;
            }
        }

        let events = events.lock().unwrap().clone();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], SyncEvent::SyncSuccess);
        assert_eq!(
            events[1],
            SyncEvent::CooldownEnd(CooldownEnd::FromSyncSuccess)
        );

        handle.await.unwrap();
    }

    #[tokio::test]
    async fn sync_failure() {
        let dir = Dir::create_temp_dir("spawn").await.unwrap();
        let auth_client = Arc::new(MockAuthClient::default());
        let (token_mngr, _) = create_token_manager(&dir, auth_client.clone()).await;

        let http_client = Arc::new(MockConfigInstancesClient::default());
        http_client.set_list_all_config_instances(|| {
            Err(HTTPErr::MockErr(Box::new(MockErr {
                is_network_connection_error: true,
            })))
        });
        http_client.set_update_config_instance(|| {
            Err(HTTPErr::MockErr(Box::new(MockErr {
                is_network_connection_error: true,
            })))
        });

        // create the caches
        let (cfg_inst_cache, _) =
            ConfigInstanceCache::spawn(16, dir.file("cfg_inst_cache.json"), 1000)
                .await
                .unwrap();
        let (cfg_inst_content_cache, _) =
            ConfigInstanceContentCache::spawn(16, dir.subdir("cfg_inst_content_cache"), 1000)
                .await
                .unwrap();

        let cooldown_options = CooldownOptions {
            base_secs: 1,
            ..CooldownOptions::default()
        };
        let (syncer, _) = spawn(
            32,
            SyncerArgs {
                device_id: "device_id".to_string(),
                http_client: http_client.clone(),
                token_mngr: Arc::new(token_mngr),
                cfg_inst_cache: Arc::new(cfg_inst_cache),
                cfg_inst_content_cache: Arc::new(cfg_inst_content_cache),
                deployment_dir: dir.clone(),
                fsm_settings: fsm::Settings::default(),
                cooldown_options,
            },
        )
        .unwrap();

        let mut subscriber = syncer.subscribe().await.unwrap();
        let events = Arc::new(Mutex::new(vec![]));

        let mut subscriber_for_spawn = subscriber.clone();
        let events_for_spawn = events.clone();
        let handle = tokio::spawn(async move {
            // expect two events: 1. not synced and then 2. cooldown ended
            for _ in 0..2 {
                subscriber_for_spawn.changed().await.unwrap();
                events_for_spawn
                    .lock()
                    .unwrap()
                    .push(subscriber_for_spawn.borrow().clone());
            }
        });

        syncer.sync().await.unwrap_err();
        // Wait for the cooldown to end
        loop {
            subscriber.changed().await.unwrap();
            let event = subscriber.borrow().clone();
            if matches!(event, SyncEvent::CooldownEnd(CooldownEnd::FromSyncFailure)) {
                break;
            }
        }

        let events = events.lock().unwrap().clone();
        assert_eq!(events.len(), 2);
        assert_eq!(
            events[0],
            SyncEvent::SyncFailed(SyncFailure {
                is_network_connection_error: true,
            })
        );
        assert_eq!(
            events[1],
            SyncEvent::CooldownEnd(CooldownEnd::FromSyncFailure)
        );

        handle.await.unwrap();
    }
}

// get_sync_state, is_in_cooldown, get_cooldown_ends_at
// sync function tests below

pub mod sync {
    use super::*;

    #[tokio::test]
    async fn pull_deploy_and_push() {
        let dir = Dir::create_temp_dir("spawn").await.unwrap();
        let auth_client = Arc::new(MockAuthClient::default());
        let (token_mngr, _) = create_token_manager(&dir, auth_client.clone()).await;

        // define the new config instance
        let id = "new_instance".to_string();
        let new_instance_data = serde_json::json!({"id": id});
        let new_instance = openapi_client::models::ConfigInstance {
            id: id.clone(),
            target_status: openapi_client::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_DEPLOYED,
            content: Some(new_instance_data.clone()),
            ..Default::default()
        };
        let http_client = Arc::new(MockConfigInstancesClient::default());
        let new_instance_cloned = new_instance.clone();
        http_client.set_list_all_config_instances(move || Ok(vec![new_instance_cloned.clone()]));

        // create the caches
        let (cfg_inst_cache, _) =
            ConfigInstanceCache::spawn(16, dir.file("cfg_inst_cache.json"), 1000)
                .await
                .unwrap();
        let (cfg_inst_content_cache, _) =
            ConfigInstanceContentCache::spawn(16, dir.subdir("cfg_inst_content_cache"), 1000)
                .await
                .unwrap();
        let cfg_inst_cache = Arc::new(cfg_inst_cache);
        let cfg_inst_content_cache = Arc::new(cfg_inst_content_cache);

        let cooldown_options = CooldownOptions {
            base_secs: 10,
            ..CooldownOptions::default()
        };
        let (syncer, _) = spawn(
            32,
            SyncerArgs {
                device_id: "device_id".to_string(),
                http_client: http_client.clone(),
                token_mngr: Arc::new(token_mngr),
                cfg_inst_cache: cfg_inst_cache.clone(),
                cfg_inst_content_cache: cfg_inst_content_cache.clone(),
                deployment_dir: dir.clone(),
                fsm_settings: fsm::Settings::default(),
                cooldown_options,
            },
        )
        .unwrap();

        let before = Utc::now();
        syncer.sync().await.unwrap();
        let after = Utc::now();

        // check the metadata cache has the new config instance
        let cache_cfg_inst = cfg_inst_cache.read(id.clone()).await.unwrap();
        assert_eq!(cache_cfg_inst.activity_status, ActivityStatus::Deployed);

        // check the content cache has the new config instance content
        let cache_cfg_inst_content = cfg_inst_content_cache.read(id.clone()).await.unwrap();
        assert_eq!(cache_cfg_inst_content, new_instance_data);

        // check that the metadata cache isn't dirty
        let unsynced_entries = cfg_inst_cache.get_dirty_entries().await.unwrap();
        assert_eq!(unsynced_entries.len(), 0);

        // check the sync state
        let state = syncer.get_sync_state().await.unwrap();
        assert_eq!(
            syncer.get_cooldown_ends_at().await.unwrap(),
            state.cooldown_ends_at
        );
        assert!(state.last_sync_attempted_at > before);
        assert!(state.last_sync_attempted_at < after);
        assert!(state.last_successful_sync_at > before);
        assert!(state.last_successful_sync_at < after);
        let base_cooldown_duration = TimeDelta::seconds(cooldown_options.base_secs);
        assert!(state.cooldown_ends_at > before + base_cooldown_duration);
        assert!(state.cooldown_ends_at < after + base_cooldown_duration);
        assert_eq!(state.err_streak, 0);
    }

    #[tokio::test]
    async fn network_error() {
        let dir = Dir::create_temp_dir("spawn").await.unwrap();
        let auth_client = Arc::new(MockAuthClient::default());
        let (token_mngr, _) = create_token_manager(&dir, auth_client.clone()).await;

        let http_client = Arc::new(MockConfigInstancesClient::default());
        http_client.set_list_all_config_instances(|| {
            Err(HTTPErr::MockErr(Box::new(MockErr {
                is_network_connection_error: true,
            })))
        });
        http_client.set_update_config_instance(|| {
            Err(HTTPErr::MockErr(Box::new(MockErr {
                is_network_connection_error: true,
            })))
        });

        // create the caches
        let (cfg_inst_cache, _) =
            ConfigInstanceCache::spawn(16, dir.file("cfg_inst_cache.json"), 1000)
                .await
                .unwrap();
        let (cfg_inst_content_cache, _) =
            ConfigInstanceContentCache::spawn(16, dir.subdir("cfg_inst_content_cache"), 1000)
                .await
                .unwrap();

        let cooldown_options = CooldownOptions {
            base_secs: 10,
            ..CooldownOptions::default()
        };
        let (syncer, _) = spawn(
            32,
            SyncerArgs {
                device_id: "device_id".to_string(),
                http_client: http_client.clone(),
                token_mngr: Arc::new(token_mngr),
                cfg_inst_cache: Arc::new(cfg_inst_cache),
                cfg_inst_content_cache: Arc::new(cfg_inst_content_cache),
                deployment_dir: dir.clone(),
                fsm_settings: fsm::Settings::default(),
                cooldown_options,
            },
        )
        .unwrap();

        let base_cooldown_duration = TimeDelta::seconds(cooldown_options.base_secs);
        for _ in 0..10 {
            let before = Utc::now();
            let error = syncer.sync().await.unwrap_err();
            let after = Utc::now();

            // check error type
            assert!(error.is_network_connection_error());

            // check the sync state
            let state = syncer.get_sync_state().await.unwrap();
            assert_eq!(
                syncer.get_cooldown_ends_at().await.unwrap(),
                state.cooldown_ends_at
            );
            assert!(state.last_sync_attempted_at > before);
            assert!(state.last_sync_attempted_at < after);
            assert_eq!(state.last_successful_sync_at, DateTime::<Utc>::UNIX_EPOCH);
            assert!(state.cooldown_ends_at > before + base_cooldown_duration);
            assert!(state.cooldown_ends_at < after + base_cooldown_duration);
            assert_eq!(state.err_streak, 0);

            // double check sync state functions
            assert!(syncer.is_in_cooldown().await.unwrap());

            // reset the syncer state
            #[cfg(feature = "test")]
            syncer
                .set_sync_state(SyncState {
                    cooldown_ends_at: before,
                    ..state
                })
                .await
                .unwrap();
        }
    }

    #[tokio::test]
    async fn non_network_error() {
        let dir = Dir::create_temp_dir("spawn").await.unwrap();
        let auth_client = Arc::new(MockAuthClient::default());
        let (token_mngr, _) = create_token_manager(&dir, auth_client.clone()).await;

        // all errors need to be a network connection error for the syncer to return a
        // network connection error so only set one false to test this
        let http_client = Arc::new(MockConfigInstancesClient::default());
        http_client.set_list_all_config_instances(|| {
            Err(HTTPErr::MockErr(Box::new(MockErr {
                is_network_connection_error: false,
            })))
        });
        http_client.set_update_config_instance(|| {
            Err(HTTPErr::MockErr(Box::new(MockErr {
                is_network_connection_error: false,
            })))
        });

        // create the caches
        let (cfg_inst_cache, _) =
            ConfigInstanceCache::spawn(16, dir.file("cfg_inst_cache.json"), 1000)
                .await
                .unwrap();
        let (cfg_inst_content_cache, _) =
            ConfigInstanceContentCache::spawn(16, dir.subdir("cfg_inst_content_cache"), 1000)
                .await
                .unwrap();

        let cooldown_options = CooldownOptions {
            base_secs: 10,
            ..CooldownOptions::default()
        };
        let (syncer, _) = spawn(
            32,
            SyncerArgs {
                device_id: "device_id".to_string(),
                http_client: http_client.clone(),
                token_mngr: Arc::new(token_mngr),
                cfg_inst_cache: Arc::new(cfg_inst_cache),
                cfg_inst_content_cache: Arc::new(cfg_inst_content_cache),
                deployment_dir: dir.clone(),
                fsm_settings: fsm::Settings::default(),
                cooldown_options,
            },
        )
        .unwrap();

        for i in 0..10 {
            let before = Utc::now();
            let error = syncer.sync().await.unwrap_err();
            let after = Utc::now();

            // check error type
            assert!(!error.is_network_connection_error());

            // check the sync state
            let state = syncer.get_sync_state().await.unwrap();
            assert_eq!(
                syncer.get_cooldown_ends_at().await.unwrap(),
                state.cooldown_ends_at
            );
            assert!(state.last_sync_attempted_at > before);
            assert!(state.last_sync_attempted_at < after);
            assert_eq!(state.last_successful_sync_at, DateTime::<Utc>::UNIX_EPOCH);
            let cooldown_secs = calc_exp_backoff(
                cooldown_options.base_secs,
                cooldown_options.growth_factor,
                i + 1,
                cooldown_options.max_secs,
            );
            let cooldown_duration = TimeDelta::seconds(cooldown_secs);
            assert!(state.cooldown_ends_at > before + cooldown_duration);
            assert!(state.cooldown_ends_at < after + cooldown_duration);
            assert_eq!(state.err_streak, i + 1);

            // double check sync state functions
            assert!(syncer.is_in_cooldown().await.unwrap());

            // reset the syncer state
            #[cfg(feature = "test")]
            syncer
                .set_sync_state(SyncState {
                    cooldown_ends_at: before,
                    ..state
                })
                .await
                .unwrap();
        }
    }

    #[tokio::test]
    async fn non_network_error_to_network_error_to_recovery() {
        let dir = Dir::create_temp_dir("spawn").await.unwrap();
        let auth_client = Arc::new(MockAuthClient::default());
        let (token_mngr, _) = create_token_manager(&dir, auth_client.clone()).await;

        let http_client = Arc::new(MockConfigInstancesClient::default());
        http_client.set_list_all_config_instances(|| {
            Err(HTTPErr::MockErr(Box::new(MockErr {
                is_network_connection_error: false,
            })))
        });
        http_client.set_update_config_instance(|| {
            Err(HTTPErr::MockErr(Box::new(MockErr {
                is_network_connection_error: false,
            })))
        });

        // create the caches
        let (cfg_inst_cache, _) =
            ConfigInstanceCache::spawn(16, dir.file("cfg_inst_cache.json"), 1000)
                .await
                .unwrap();
        let (cfg_inst_content_cache, _) =
            ConfigInstanceContentCache::spawn(16, dir.subdir("cfg_inst_content_cache"), 1000)
                .await
                .unwrap();

        let cooldown_options = CooldownOptions {
            base_secs: 10,
            ..CooldownOptions::default()
        };
        let (syncer, _) = spawn(
            32,
            SyncerArgs {
                device_id: "device_id".to_string(),
                http_client: http_client.clone(),
                token_mngr: Arc::new(token_mngr),
                cfg_inst_cache: Arc::new(cfg_inst_cache),
                cfg_inst_content_cache: Arc::new(cfg_inst_content_cache),
                deployment_dir: dir.clone(),
                fsm_settings: fsm::Settings::default(),
                cooldown_options,
            },
        )
        .unwrap();

        // non-network connection errors
        for i in 0..10 {
            let before = Utc::now();
            let error = syncer.sync().await.unwrap_err();
            let after = Utc::now();

            // check error type
            assert!(!error.is_network_connection_error());

            // check the sync state
            let state = syncer.get_sync_state().await.unwrap();
            assert_eq!(
                syncer.get_cooldown_ends_at().await.unwrap(),
                state.cooldown_ends_at
            );
            assert!(state.last_sync_attempted_at > before);
            assert!(state.last_sync_attempted_at < after);
            assert_eq!(state.last_successful_sync_at, DateTime::<Utc>::UNIX_EPOCH);
            let cooldown_secs = calc_exp_backoff(
                cooldown_options.base_secs,
                cooldown_options.growth_factor,
                i + 1,
                cooldown_options.max_secs,
            );
            let cooldown_duration = TimeDelta::seconds(cooldown_secs);
            assert!(state.cooldown_ends_at > before + cooldown_duration);
            assert!(state.cooldown_ends_at < after + cooldown_duration);
            assert_eq!(state.err_streak, i + 1);

            // double check sync state functions
            assert!(syncer.is_in_cooldown().await.unwrap());

            // reset the syncer state
            #[cfg(feature = "test")]
            syncer
                .set_sync_state(SyncState {
                    cooldown_ends_at: before,
                    ..state
                })
                .await
                .unwrap();
        }

        // set the http client to return a network connection error
        http_client.set_list_all_config_instances(|| {
            Err(HTTPErr::MockErr(Box::new(MockErr {
                is_network_connection_error: true,
            })))
        });
        http_client.set_update_config_instance(|| {
            Err(HTTPErr::MockErr(Box::new(MockErr {
                is_network_connection_error: true,
            })))
        });

        // non-network connection errors
        let cur_err_streak = 10;
        let base_cooldown_duration = TimeDelta::seconds(cooldown_options.base_secs);
        for _ in 0..10 {
            let before = Utc::now();
            let error = syncer.sync().await.unwrap_err();
            let after = Utc::now();

            // check error type
            assert!(error.is_network_connection_error());

            // check the sync state
            let state = syncer.get_sync_state().await.unwrap();
            assert!(state.last_sync_attempted_at > before);
            assert!(state.last_sync_attempted_at < after);
            assert_eq!(state.last_successful_sync_at, DateTime::<Utc>::UNIX_EPOCH);
            assert!(state.cooldown_ends_at > before + base_cooldown_duration);
            assert!(state.cooldown_ends_at < after + base_cooldown_duration);
            assert_eq!(state.err_streak, cur_err_streak);

            // double check sync state functions
            assert!(syncer.is_in_cooldown().await.unwrap());

            // reset the syncer state
            #[cfg(feature = "test")]
            syncer
                .set_sync_state(SyncState {
                    cooldown_ends_at: before,
                    ..state
                })
                .await
                .unwrap();
        }

        // set the http client to not return an error
        http_client.set_list_all_config_instances(|| Ok(vec![]));
        http_client.set_update_config_instance(|| {
            Ok(openapi_client::models::ConfigInstance {
                ..Default::default()
            })
        });

        // recovery
        let base_cooldown_duration = TimeDelta::seconds(cooldown_options.base_secs);
        for _ in 0..10 {
            let before = Utc::now();
            syncer.sync().await.unwrap();
            let after = Utc::now();

            // check the sync state
            let state = syncer.get_sync_state().await.unwrap();
            assert_eq!(
                syncer.get_cooldown_ends_at().await.unwrap(),
                state.cooldown_ends_at
            );
            assert!(state.last_sync_attempted_at > before);
            assert!(state.last_sync_attempted_at < after);
            assert!(state.last_successful_sync_at > before);
            assert!(state.last_successful_sync_at < after);
            assert!(state.cooldown_ends_at > before + base_cooldown_duration);
            assert!(state.cooldown_ends_at < after + base_cooldown_duration);
            assert_eq!(state.err_streak, 0);

            // double check sync state functions
            assert!(syncer.is_in_cooldown().await.unwrap());

            // reset the syncer state
            #[cfg(feature = "test")]
            syncer
                .set_sync_state(SyncState {
                    cooldown_ends_at: before,
                    ..state
                })
                .await
                .unwrap();
        }
    }

    #[tokio::test]
    async fn in_cooldown_error() {
        let dir = Dir::create_temp_dir("spawn").await.unwrap();
        let auth_client = Arc::new(MockAuthClient::default());
        let (token_mngr, _) = create_token_manager(&dir, auth_client.clone()).await;
        let http_client = Arc::new(MockConfigInstancesClient::default());

        // create the caches
        let (cfg_inst_cache, _) =
            ConfigInstanceCache::spawn(16, dir.file("cfg_inst_cache.json"), 1000)
                .await
                .unwrap();
        let (cfg_inst_content_cache, _) =
            ConfigInstanceContentCache::spawn(16, dir.subdir("cfg_inst_content_cache"), 1000)
                .await
                .unwrap();

        let cooldown_options = CooldownOptions {
            base_secs: 10,
            ..CooldownOptions::default()
        };
        let (syncer, _) = spawn(
            32,
            SyncerArgs {
                device_id: "device_id".to_string(),
                http_client: http_client.clone(),
                token_mngr: Arc::new(token_mngr),
                cfg_inst_cache: Arc::new(cfg_inst_cache),
                cfg_inst_content_cache: Arc::new(cfg_inst_content_cache),
                deployment_dir: dir.clone(),
                fsm_settings: fsm::Settings::default(),
                cooldown_options,
            },
        )
        .unwrap();

        // set the syncer state to be in cooldown
        #[cfg(feature = "test")]
        syncer
            .set_sync_state(SyncState {
                last_sync_attempted_at: DateTime::<Utc>::UNIX_EPOCH,
                last_successful_sync_at: DateTime::<Utc>::UNIX_EPOCH,
                cooldown_ends_at: Utc::now() + TimeDelta::seconds(10),
                err_streak: 0,
            })
            .await
            .unwrap();

        let error = syncer.sync().await.unwrap_err();
        assert!(matches!(error, SyncErr::InCooldownErr(_)));
    }
}

pub mod sync_if_not_in_cooldown {
    use super::*;

    #[tokio::test]
    async fn sync_if_not_in_cooldown() {
        let dir = Dir::create_temp_dir("spawn").await.unwrap();
        let auth_client = Arc::new(MockAuthClient::default());
        let (token_mngr, _) = create_token_manager(&dir, auth_client.clone()).await;
        let http_client = Arc::new(MockConfigInstancesClient::default());

        // create the caches
        let (cfg_inst_cache, _) =
            ConfigInstanceCache::spawn(16, dir.file("cfg_inst_cache.json"), 1000)
                .await
                .unwrap();
        let (cfg_inst_content_cache, _) =
            ConfigInstanceContentCache::spawn(16, dir.subdir("cfg_inst_content_cache"), 1000)
                .await
                .unwrap();

        let cooldown_options = CooldownOptions {
            base_secs: 10,
            ..CooldownOptions::default()
        };
        let (syncer, _) = spawn(
            32,
            SyncerArgs {
                device_id: "device_id".to_string(),
                http_client: http_client.clone(),
                token_mngr: Arc::new(token_mngr),
                cfg_inst_cache: Arc::new(cfg_inst_cache),
                cfg_inst_content_cache: Arc::new(cfg_inst_content_cache),
                deployment_dir: dir.clone(),
                fsm_settings: fsm::Settings::default(),
                cooldown_options,
            },
        )
        .unwrap();

        // set the syncer state to be in cooldown
        #[cfg(feature = "test")]
        syncer
            .set_sync_state(SyncState {
                last_sync_attempted_at: DateTime::<Utc>::UNIX_EPOCH,
                last_successful_sync_at: DateTime::<Utc>::UNIX_EPOCH,
                cooldown_ends_at: Utc::now() + TimeDelta::seconds(10),
                err_streak: 0,
            })
            .await
            .unwrap();

        syncer.sync_if_not_in_cooldown().await.unwrap();
        assert_eq!(
            syncer
                .get_sync_state()
                .await
                .unwrap()
                .last_sync_attempted_at,
            DateTime::<Utc>::UNIX_EPOCH
        );

        // set the syncer state to be in cooldown
        #[cfg(feature = "test")]
        syncer
            .set_sync_state(SyncState {
                last_sync_attempted_at: DateTime::<Utc>::UNIX_EPOCH,
                last_successful_sync_at: DateTime::<Utc>::UNIX_EPOCH,
                cooldown_ends_at: DateTime::<Utc>::UNIX_EPOCH,
                err_streak: 0,
            })
            .await
            .unwrap();

        let before = Utc::now();
        syncer.sync_if_not_in_cooldown().await.unwrap();
        let after = Utc::now();
        assert!(
            syncer
                .get_sync_state()
                .await
                .unwrap()
                .last_sync_attempted_at
                > before
        );
        assert!(
            syncer
                .get_sync_state()
                .await
                .unwrap()
                .last_sync_attempted_at
                < after
        );
    }
}
