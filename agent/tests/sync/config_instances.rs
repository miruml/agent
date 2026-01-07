// standard crates
use std::sync::Arc;

// internal crates
use miru_agent::crud::prelude::*;
use miru_agent::deploy::fsm;
use miru_agent::filesys::dir::Dir;
use miru_agent::http::errors::*;
use miru_agent::models::config_instance::{
    ActivityStatus, ConfigInstance, ErrorStatus, TargetStatus,
};
use miru_agent::storage::config_instances::{ConfigInstanceCache, ConfigInstanceContentCache};
use miru_agent::sync::config_instances::{pull, push, sync};

use crate::http::mock::{CfgInstsCall, MockCfgInstsClient};

use openapi_client::models::UpdateConfigInstanceRequest;

// external crates
use serde_json::json;

pub mod sync {
    use super::*;

    #[tokio::test]
    async fn pull_deploy_and_push() {
        let dir = Dir::create_temp_dir("spawn").await.unwrap();

        // define the new config instance
        let id = "new_instance".to_string();
        let new_instance_data = serde_json::json!({"id": id});
        let new_instance = openapi_client::models::ConfigInstance {
            id: id.clone(),
            target_status: openapi_client::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_DEPLOYED,
            activity_status: openapi_client::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_QUEUED,
            content: Some(new_instance_data.clone()),
            relative_filepath: "/test/filepath".to_string(),
            ..Default::default()
        };
        let http_client = MockCfgInstsClient::default();
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

        sync(
            &cfg_inst_cache,
            &cfg_inst_content_cache,
            &http_client,
            "device_id",
            &dir,
            &fsm::Settings::default(),
            "token",
        )
        .await
        .unwrap();

        // check the metadata cache has the new config instance
        let cache_cfg_inst = cfg_inst_cache.read(id.clone()).await.unwrap();
        assert_eq!(cache_cfg_inst.activity_status, ActivityStatus::Deployed);

        // check the content cache has the new config instance content
        let cache_cfg_inst_content = cfg_inst_content_cache.read(id.clone()).await.unwrap();
        assert_eq!(cache_cfg_inst_content, new_instance_data);

        // check that the http client was called to update the config instance
        assert_eq!(http_client.num_update_config_instance_calls(), 1);

        // check that the metadata cache isn't dirty
        let unsynced_entries = cfg_inst_cache.get_dirty_entries().await.unwrap();
        assert_eq!(unsynced_entries.len(), 0);
    }

    #[tokio::test]
    async fn pull_failure() {
        let dir = Dir::create_temp_dir("spawn").await.unwrap();

        // pull fails
        let http_client = MockCfgInstsClient::default();
        http_client.set_list_all_config_instances(move || {
            Err(HTTPErr::MockErr(Box::new(MockErr {
                is_network_connection_error: false,
            })))
        });

        // create the caches with undeployed config instance
        let id = "new_instance".to_string();
        let content = serde_json::json!({"id": id});
        let cfg_inst = ConfigInstance {
            id: id.clone(),
            target_status: TargetStatus::Deployed,
            activity_status: ActivityStatus::Queued,
            relative_filepath: "/test/filepath".to_string(),
            ..Default::default()
        };
        let (cfg_inst_cache, _) =
            ConfigInstanceCache::spawn(16, dir.file("cfg_inst_cache.json"), 1000)
                .await
                .unwrap();
        cfg_inst_cache
            .write(id.clone(), cfg_inst.clone(), |_, _| true, true)
            .await
            .unwrap();
        let (cfg_inst_content_cache, _) =
            ConfigInstanceContentCache::spawn(16, dir.subdir("cfg_inst_content_cache"), 1000)
                .await
                .unwrap();
        cfg_inst_content_cache
            .write(id.clone(), content.clone(), |_, _| false, true)
            .await
            .unwrap();
        let cfg_inst_cache = Arc::new(cfg_inst_cache);
        let cfg_inst_content_cache = Arc::new(cfg_inst_content_cache);

        // sync should fail from the pull but still execute the apply and push
        sync(
            &cfg_inst_cache,
            &cfg_inst_content_cache,
            &http_client,
            "device_id",
            &dir,
            &fsm::Settings::default(),
            "token",
        )
        .await
        .unwrap_err();

        // check the config instance was deployed
        let cache_cfg_inst = cfg_inst_cache.read(id.clone()).await.unwrap();
        assert_eq!(cache_cfg_inst.activity_status, ActivityStatus::Deployed);

        // check that the http client was called to update the config instance
        assert_eq!(http_client.num_update_config_instance_calls(), 1);

        // check that the metadata cache isn't dirty
        let unsynced_entries = cfg_inst_cache.get_dirty_entries().await.unwrap();
        assert_eq!(unsynced_entries.len(), 0);
    }

    #[tokio::test]
    async fn push_failure() {
        let dir = Dir::create_temp_dir("spawn").await.unwrap();

        // pull fails
        let http_client = MockCfgInstsClient::default();
        http_client.set_update_config_instance(move || {
            Err(HTTPErr::MockErr(Box::new(MockErr {
                is_network_connection_error: false,
            })))
        });

        // create the caches with undeployed config instance
        let id = "new_instance".to_string();
        let content = serde_json::json!({"id": id});
        let cfg_inst = ConfigInstance {
            id: id.clone(),
            target_status: TargetStatus::Deployed,
            activity_status: ActivityStatus::Queued,
            relative_filepath: "/test/filepath".to_string(),
            ..Default::default()
        };
        let (cfg_inst_cache, _) =
            ConfigInstanceCache::spawn(16, dir.file("cfg_inst_cache.json"), 1000)
                .await
                .unwrap();
        cfg_inst_cache
            .write(id.clone(), cfg_inst.clone(), |_, _| true, true)
            .await
            .unwrap();
        let (cfg_inst_content_cache, _) =
            ConfigInstanceContentCache::spawn(16, dir.subdir("cfg_inst_content_cache"), 1000)
                .await
                .unwrap();
        cfg_inst_content_cache
            .write(id.clone(), content.clone(), |_, _| false, true)
            .await
            .unwrap();
        let cfg_inst_cache = Arc::new(cfg_inst_cache);
        let cfg_inst_content_cache = Arc::new(cfg_inst_content_cache);

        // sync should fail from the push but still execute the apply the config instances
        sync(
            &cfg_inst_cache,
            &cfg_inst_content_cache,
            &http_client,
            "device_id",
            &dir,
            &fsm::Settings::default(),
            "token",
        )
        .await
        .unwrap_err();

        // check the config instance was deployed
        let cache_cfg_inst = cfg_inst_cache.read(id.clone()).await.unwrap();
        assert_eq!(cache_cfg_inst.activity_status, ActivityStatus::Deployed);

        // check that the http client was called to update the config instance
        assert_eq!(http_client.num_update_config_instance_calls(), 1);

        // check that the metadata cache is dirty
        let unsynced_entries = cfg_inst_cache.get_dirty_entries().await.unwrap();
        assert_eq!(unsynced_entries.len(), 1);
    }
}

pub mod pull {
    use super::*;

    #[tokio::test]
    async fn no_instances() {
        // define the caches
        let dir = Dir::create_temp_dir("apply").await.unwrap();
        let (cfg_inst_cache, _) = ConfigInstanceCache::spawn(16, dir.file("metadata.json"), 1000)
            .await
            .unwrap();
        let (cfg_inst_content_cache, _) =
            ConfigInstanceContentCache::spawn(16, dir.subdir("instances"), 1000)
                .await
                .unwrap();

        // define the mock http client
        let http_client = MockCfgInstsClient::default();

        // pull the config instances
        pull(
            &cfg_inst_cache,
            &cfg_inst_content_cache,
            &http_client,
            "device_id",
            "token",
        )
        .await
        .unwrap();

        // assert the caches are still empty
        assert_eq!(cfg_inst_cache.size().await.unwrap(), 0);
        assert_eq!(cfg_inst_content_cache.size().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn one_unknown_instance() {
        // define the caches
        let dir = Dir::create_temp_dir("apply").await.unwrap();
        let (cfg_inst_cache, _) = ConfigInstanceCache::spawn(16, dir.file("metadata.json"), 1000)
            .await
            .unwrap();
        let (cfg_inst_content_cache, _) =
            ConfigInstanceContentCache::spawn(16, dir.subdir("instances"), 1000)
                .await
                .unwrap();

        // define the mock http client
        let http_client = MockCfgInstsClient::default();
        let cfg_inst_content = json!({
            "cfg_inst1": {
                "metadata": "metadata1",
                "content": "content1",
            }
        });
        let id = "instance1".to_string();
        let result = vec![openapi_client::models::ConfigInstance {
            id: id.clone(),
            content: Some(cfg_inst_content.clone()),
            ..Default::default()
        }];
        let result_cloned = result.clone();
        http_client.set_list_all_config_instances(move || Ok(result_cloned.clone()));

        // pull the config instances
        pull(
            &cfg_inst_cache,
            &cfg_inst_content_cache,
            &http_client,
            "device_id",
            "token",
        )
        .await
        .unwrap();

        // check the metadata cache
        assert_eq!(cfg_inst_cache.size().await.unwrap(), 1);
        let expected = ConfigInstance::from_backend(result[0].clone());
        let actual = cfg_inst_cache.read(id.clone()).await.unwrap();
        assert_eq!(expected, actual);

        // check the config instance content cache
        assert_eq!(cfg_inst_content_cache.size().await.unwrap(), 1);
        let expected = cfg_inst_content;
        let actual = cfg_inst_content_cache.read(id.clone()).await.unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn n_unknown_instances() {
        // define the caches
        let dir = Dir::create_temp_dir("apply").await.unwrap();
        let (cfg_inst_cache, _) = ConfigInstanceCache::spawn(16, dir.file("metadata.json"), 1000)
            .await
            .unwrap();
        let (cfg_inst_content_cache, _) =
            ConfigInstanceContentCache::spawn(16, dir.subdir("instances"), 1000)
                .await
                .unwrap();

        // define the mock http client
        let n = 10;
        let http_client = MockCfgInstsClient::default();
        let mut instance_datas = Vec::new();
        let mut metadatas = Vec::new();
        for i in 0..n {
            let cfg_inst_content = json!({
                "cfg_inst1": {
                    "metadata": format!("metadata{}", i),
                    "content": format!("content{}", i),
                }
            });
            instance_datas.push(cfg_inst_content.clone());

            let id = format!("cfg_inst{i}");
            let backend_instance = openapi_client::models::ConfigInstance {
                id: id.clone(),
                content: Some(cfg_inst_content.clone()),
                ..Default::default()
            };
            metadatas.push(backend_instance);
        }
        let metadatas_cloned = metadatas.clone();
        http_client.set_list_all_config_instances(move || Ok(metadatas_cloned.clone()));

        // pull the config instances
        pull(
            &cfg_inst_cache,
            &cfg_inst_content_cache,
            &http_client,
            "device_id",
            "token",
        )
        .await
        .unwrap();

        // check the metadata cache
        assert_eq!(cfg_inst_cache.size().await.unwrap(), n);
        for metadata in metadatas.iter() {
            let id = metadata.id.clone();
            let expected = ConfigInstance::from_backend(metadata.clone());
            let actual = cfg_inst_cache.read(id.clone()).await.unwrap();
            assert_eq!(expected, actual);
        }

        // check the config instance content cache
        assert_eq!(cfg_inst_content_cache.size().await.unwrap(), n);
        for (i, cfg_inst_content) in instance_datas.iter().enumerate() {
            let id = format!("cfg_inst{i}");
            let expected = cfg_inst_content.clone();
            let actual = cfg_inst_content_cache.read(id.clone()).await.unwrap();
            assert_eq!(expected, actual);
        }
    }

    #[tokio::test]
    async fn one_instance_with_updated_target_status() {
        // define the existing config instance
        let id = "instance1".to_string();
        let existing_instance = ConfigInstance {
            id: id.clone(),
            target_status: TargetStatus::Deployed,
            activity_status: ActivityStatus::Deployed,
            ..Default::default()
        };
        let cfg_inst_content = json!({
            "cfg_inst1": {
                "metadata": "metadata1",
                "content": "content1",
            }
        });

        // define the caches
        let dir = Dir::create_temp_dir("apply").await.unwrap();
        let (cfg_inst_cache, _) = ConfigInstanceCache::spawn(16, dir.file("metadata.json"), 1000)
            .await
            .unwrap();
        cfg_inst_cache
            .write(id.clone(), existing_instance.clone(), |_, _| false, true)
            .await
            .unwrap();
        let (cfg_inst_content_cache, _) =
            ConfigInstanceContentCache::spawn(16, dir.subdir("instances"), 1000)
                .await
                .unwrap();
        cfg_inst_content_cache
            .write(id.clone(), cfg_inst_content.clone(), |_, _| false, true)
            .await
            .unwrap();

        // define the mock http client
        let http_client = MockCfgInstsClient::default();
        let result = vec![
            openapi_client::models::ConfigInstance {
                id: id.clone(),
                content: Some(cfg_inst_content.clone()),
                target_status: openapi_client::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_REMOVED,
                // use a different activity status from the server to ensure that the
                // activity status is NOT updated when updating new target statuses
                activity_status: openapi_client::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_QUEUED,
                ..Default::default()
            }
        ];
        let result_cloned = result.clone();
        http_client.set_list_all_config_instances(move || Ok(result_cloned.clone()));

        // pull the config instances
        pull(
            &cfg_inst_cache,
            &cfg_inst_content_cache,
            &http_client,
            "device_id",
            "token",
        )
        .await
        .unwrap();

        // check the metadata cache
        assert_eq!(cfg_inst_cache.size().await.unwrap(), 1);
        let expected = ConfigInstance {
            target_status: TargetStatus::Removed,
            ..existing_instance
        };
        let actual = cfg_inst_cache.read(id.clone()).await.unwrap();
        assert_eq!(expected, actual);

        // check the config instance content cache
        assert_eq!(cfg_inst_content_cache.size().await.unwrap(), 1);
        let expected = cfg_inst_content;
        let actual = cfg_inst_content_cache.read(id.clone()).await.unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn one_instance_up_to_date() {
        // define the existing config instance
        let id = "instance1".to_string();
        let existing_instance = ConfigInstance {
            id: id.clone(),
            target_status: TargetStatus::Deployed,
            ..Default::default()
        };
        let cfg_inst_content = json!({
            "cfg_inst1": {
                "metadata": "metadata1",
                "content": "content1",
            }
        });

        // define the caches
        let dir = Dir::create_temp_dir("apply").await.unwrap();
        let (cfg_inst_cache, _) = ConfigInstanceCache::spawn(16, dir.file("metadata.json"), 1000)
            .await
            .unwrap();
        cfg_inst_cache
            .write(id.clone(), existing_instance.clone(), |_, _| false, true)
            .await
            .unwrap();
        let (cfg_inst_content_cache, _) =
            ConfigInstanceContentCache::spawn(16, dir.subdir("instances"), 1000)
                .await
                .unwrap();
        cfg_inst_content_cache
            .write(id.clone(), cfg_inst_content.clone(), |_, _| false, true)
            .await
            .unwrap();

        // define the mock http client
        let http_client = MockCfgInstsClient::default();
        let result = vec![
            openapi_client::models::ConfigInstance {
                id: id.clone(),
                content: Some(cfg_inst_content.clone()),
                target_status: openapi_client::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_DEPLOYED,
                ..Default::default()
            }
        ];
        let result_cloned = result.clone();
        http_client.set_list_all_config_instances(move || Ok(result_cloned.clone()));

        // pull the config instances
        pull(
            &cfg_inst_cache,
            &cfg_inst_content_cache,
            &http_client,
            "device_id",
            "token",
        )
        .await
        .unwrap();

        // check the metadata cache
        assert_eq!(cfg_inst_cache.size().await.unwrap(), 1);
        let expected = existing_instance;
        let actual = cfg_inst_cache.read(id.clone()).await.unwrap();
        assert_eq!(expected, actual);

        // check the config instance content cache
        assert_eq!(cfg_inst_content_cache.size().await.unwrap(), 1);
        let expected = cfg_inst_content;
        let actual = cfg_inst_content_cache.read(id.clone()).await.unwrap();
        assert_eq!(expected, actual);
    }
}

pub mod push {
    use super::*;

    #[tokio::test]
    async fn no_instances() {
        // define the caches
        let dir = Dir::create_temp_dir("apply").await.unwrap();
        let (metadata_cache, _) = ConfigInstanceCache::spawn(16, dir.file("metadata.json"), 1000)
            .await
            .unwrap();

        // define the mock http client
        let http_client = MockCfgInstsClient::default();

        // push the config instances
        push(&metadata_cache, &http_client, "token").await.unwrap();

        // check the history
        assert_eq!(http_client.num_update_config_instance_calls(), 0);
    }

    #[tokio::test]
    async fn one_synced_instance() {
        // define the caches
        let dir = Dir::create_temp_dir("apply").await.unwrap();
        let (metadata_cache, _) = ConfigInstanceCache::spawn(16, dir.file("metadata.json"), 1000)
            .await
            .unwrap();

        // create the config instance
        let cfg_inst = ConfigInstance::default();
        metadata_cache
            .write(cfg_inst.id.clone(), cfg_inst, |_, _| false, true)
            .await
            .unwrap();

        // define the mock http client
        let http_client = MockCfgInstsClient::default();

        // push the config instances
        push(&metadata_cache, &http_client, "token").await.unwrap();

        // check the history
        assert_eq!(http_client.num_update_config_instance_calls(), 0);
    }

    #[tokio::test]
    async fn one_unsynced_instance() {
        // define the caches
        let dir = Dir::create_temp_dir("apply").await.unwrap();
        let (metadata_cache, _) = ConfigInstanceCache::spawn(16, dir.file("metadata.json"), 1000)
            .await
            .unwrap();

        // create the config instance
        let cfg_inst = ConfigInstance {
            activity_status: ActivityStatus::Deployed,
            error_status: ErrorStatus::Retrying,
            ..Default::default()
        };
        metadata_cache
            .write(cfg_inst.id.clone(), cfg_inst, |_, _| true, true)
            .await
            .unwrap();

        // define the mock http client
        let http_client = MockCfgInstsClient::default();

        // push the config instances
        push(&metadata_cache, &http_client, "token").await.unwrap();

        // check the history
        assert_eq!(http_client.num_update_config_instance_calls(), 1);
        let actual = match &http_client.get_calls()[0] {
            CfgInstsCall::UpdateConfigInstance(request) => request.clone(),
            _ => panic!("Expected UpdateConfigInstance call"),
        };
        let expected = UpdateConfigInstanceRequest {
            activity_status: Some(openapi_client::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_DEPLOYED),
            error_status: Some(openapi_client::models::ConfigInstanceErrorStatus::CONFIG_INSTANCE_ERROR_STATUS_RETRYING),
        };
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn update_config_instance_errors() {
        // define the caches
        let dir = Dir::create_temp_dir("apply").await.unwrap();
        let (metadata_cache, _) = ConfigInstanceCache::spawn(16, dir.file("metadata.json"), 1000)
            .await
            .unwrap();

        // create a few config instances
        let n_cfg_insts = 10;
        for _ in 0..n_cfg_insts {
            let cfg_inst = ConfigInstance {
                activity_status: ActivityStatus::Deployed,
                error_status: ErrorStatus::Retrying,
                ..Default::default()
            };
            metadata_cache
                .write(cfg_inst.id.clone(), cfg_inst, |_, _| true, true)
                .await
                .unwrap();
        }

        // define the mock http client
        let http_client = MockCfgInstsClient::default();
        http_client.set_update_config_instance(move || {
            Err(HTTPErr::MockErr(Box::new(MockErr {
                is_network_connection_error: false,
            })))
        });

        // push the config instances -> should fail since the updates fail but each
        // config instance should be attempted to be updated
        push(&metadata_cache, &http_client, "token")
            .await
            .unwrap_err();

        // check the history
        assert_eq!(http_client.num_update_config_instance_calls(), n_cfg_insts);

        // check that the metadata cache is dirty
        let unsynced_entries = metadata_cache.get_dirty_entries().await.unwrap();
        assert_eq!(unsynced_entries.len(), n_cfg_insts);
    }
}
