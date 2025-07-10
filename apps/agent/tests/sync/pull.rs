// internal crates
use crate::http::mock::MockConfigInstancesClient;
use config_agent::crud::prelude::*;
use config_agent::filesys::dir::Dir;
use config_agent::models::config_instance::{ConfigInstance, TargetStatus};
use config_agent::storage::config_instances::{ConfigInstanceCache, ConfigInstanceContentCache};
use config_agent::sync::pull::pull_config_instances;

// external crates
use serde_json::json;

pub mod pull_config_instances_func {
    use super::*;

    #[tokio::test]
    async fn no_instances() {
        // define the caches
        let dir = Dir::create_temp_dir("apply").await.unwrap();
        let (cfg_inst_cache, _) = ConfigInstanceCache::spawn(16, dir.file("metadata.json"), 1000)
            .await
            .unwrap();
        let (cfg_inst_content_cache, _) = ConfigInstanceContentCache::spawn(16, dir.subdir("instances"), 1000)
            .await
            .unwrap();

        // define the mock http client
        let http_client = MockConfigInstancesClient::default();

        // pull the config instances
        pull_config_instances(
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
        let (cfg_inst_content_cache, _) = ConfigInstanceContentCache::spawn(16, dir.subdir("instances"), 1000)
            .await
            .unwrap();

        // define the mock http client
        let http_client = MockConfigInstancesClient::default();
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
        pull_config_instances(
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
        let (cfg_inst_content_cache, _) = ConfigInstanceContentCache::spawn(16, dir.subdir("instances"), 1000)
            .await
            .unwrap();

        // define the mock http client
        let n = 10;
        let http_client = MockConfigInstancesClient::default();
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
        pull_config_instances(
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
        let (cfg_inst_content_cache, _) = ConfigInstanceContentCache::spawn(16, dir.subdir("instances"), 1000)
            .await
            .unwrap();
        cfg_inst_content_cache
            .write(id.clone(), cfg_inst_content.clone(), |_, _| false, true)
            .await
            .unwrap();

        // define the mock http client
        let http_client = MockConfigInstancesClient::default();
        let result = vec![
            openapi_client::models::ConfigInstance {
                id: id.clone(),
                content: Some(cfg_inst_content.clone()),
                target_status: openapi_client::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_REMOVED,
                ..Default::default()
            }
        ];
        let result_cloned = result.clone();
        http_client.set_list_all_config_instances(move || Ok(result_cloned.clone()));

        // pull the config instances
        pull_config_instances(
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
        let (cfg_inst_content_cache, _) = ConfigInstanceContentCache::spawn(16, dir.subdir("instances"), 1000)
            .await
            .unwrap();
        cfg_inst_content_cache
            .write(id.clone(), cfg_inst_content.clone(), |_, _| false, true)
            .await
            .unwrap();

        // define the mock http client
        let http_client = MockConfigInstancesClient::default();
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
        pull_config_instances(
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
