// internal crates
use config_agent::crud::prelude::*;
use config_agent::filesys::dir::Dir;
use config_agent::models::config_instance::{
    ConfigInstance,
    TargetStatus,
};
use config_agent::storage::config_instances::{
    ConfigInstanceCache,
    ConfigInstanceDataCache,
};
use config_agent::sync::pull::pull_config_instances;
use crate::http::mock::MockConfigInstancesClient;

// external crates
use serde_json::json;


pub mod pull_config_instances_func {
    use super::*;

    #[tokio::test]
    async fn no_instances() {
        // define the caches
        let dir = Dir::create_temp_dir("apply").await.unwrap();
        let (metadata_cache, _) = ConfigInstanceCache::spawn(
            16, dir.file("metadata.json"),
        ).await.unwrap();
        let (instance_cache, _) = ConfigInstanceDataCache::spawn(
            16, dir.subdir("instances"),
        ).await.unwrap();

        // define the mock http client
        let http_client = MockConfigInstancesClient::default();

        // pull the config instances
        pull_config_instances(
            &metadata_cache,
            &instance_cache,
            &http_client,
            "device_id",
            "token",
        ).await.unwrap();

        // assert the caches are still empty
        assert_eq!(metadata_cache.size().await.unwrap(), 0);
        assert_eq!(instance_cache.size().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn one_unknown_instance() {
        // define the caches
        let dir = Dir::create_temp_dir("apply").await.unwrap();
        let (metadata_cache, _) = ConfigInstanceCache::spawn(
            16, dir.file("metadata.json"),
        ).await.unwrap();
        let (instance_cache, _) = ConfigInstanceDataCache::spawn(
            16, dir.subdir("instances"),
        ).await.unwrap();

        // define the mock http client
        let mut http_client= MockConfigInstancesClient::default();
        let instance_data = json!({
            "instance1": {
                "data": "data1",
                "metadata": "metadata1",
            }
        });
        let id = "instance1".to_string();
        let result = vec![
            openapi_client::models::BackendConfigInstance {
                id: id.clone(),
                instance: Some(instance_data.clone()),
                ..Default::default()
            }
        ];
        let result_cloned = result.clone();
        http_client.set_list_all_config_instances(move || { Ok(result_cloned.clone()) });

        // pull the config instances
        pull_config_instances(
            &metadata_cache,
            &instance_cache,
            &http_client,
            "device_id",
            "token",
        ).await.unwrap();

        // check the metadata cache
        assert_eq!(metadata_cache.size().await.unwrap(), 1);
        let expected = ConfigInstance::from_backend(result[0].clone());
        let actual = metadata_cache.read(id.clone()).await.unwrap();
        assert_eq!(expected, actual);

        // check the instance data cache
        assert_eq!(instance_cache.size().await.unwrap(), 1);
        let expected = instance_data;
        let actual = instance_cache.read(id.clone()).await.unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn n_unknown_instances() {
        // define the caches
        let dir = Dir::create_temp_dir("apply").await.unwrap();
        let (metadata_cache, _) = ConfigInstanceCache::spawn(
            16, dir.file("metadata.json"),
        ).await.unwrap();
        let (instance_cache, _) = ConfigInstanceDataCache::spawn(
            16, dir.subdir("instances"),
        ).await.unwrap();

        // define the mock http client
        let n = 10;
        let mut http_client= MockConfigInstancesClient::default();
        let mut instance_datas = Vec::new();
        let mut metadatas = Vec::new();
        for i in 0..n {
            let instance_data = json!({
                "instance1": {
                    "data": format!("data{}", i),
                    "metadata": format!("metadata{}", i),
                }
            });
            instance_datas.push(instance_data.clone());

            let id = format!("instance{}", i);
            let backend_instance = openapi_client::models::BackendConfigInstance {
                id: id.clone(),
                instance: Some(instance_data.clone()),
                ..Default::default()
            };
            metadatas.push(backend_instance);
        }
        let metadatas_cloned = metadatas.clone();
        http_client.set_list_all_config_instances(move || { Ok(metadatas_cloned.clone()) });

        // pull the config instances
        pull_config_instances(
            &metadata_cache,
            &instance_cache,
            &http_client,
            "device_id",
            "token",
        ).await.unwrap();

        // check the metadata cache
        assert_eq!(metadata_cache.size().await.unwrap(), n);
        for metadata in metadatas.iter() {
            let id = metadata.id.clone();
            let expected = ConfigInstance::from_backend(metadata.clone());
            let actual = metadata_cache.read(id.clone()).await.unwrap();
            assert_eq!(expected, actual);
        }

        // check the instance data cache
        assert_eq!(instance_cache.size().await.unwrap(), n);
        for (i, instance_data) in instance_datas.iter().enumerate() {
            let id = format!("instance{}", i);
            let expected = instance_data.clone();
            let actual = instance_cache.read(id.clone()).await.unwrap();
            assert_eq!(expected, actual);
        }
    }

    #[tokio::test]
    async fn one_instance_with_updated_target_status() {
        // define the existing instance
        let id = "instance1".to_string();
        let existing_instance = ConfigInstance {
            id: id.clone(),
            target_status: TargetStatus::Deployed,
            ..Default::default()
        };
        let instance_data = json!({
            "instance1": {
                "data": "data1",
                "metadata": "metadata1",
            }
        });

        // define the caches
        let dir = Dir::create_temp_dir("apply").await.unwrap();
        let (metadata_cache, _) = ConfigInstanceCache::spawn(
            16, dir.file("metadata.json"),
        ).await.unwrap();
        metadata_cache.write(
            id.clone(), existing_instance.clone(), |_, _| false, true,
        ).await.unwrap();
        let (instance_cache, _) = ConfigInstanceDataCache::spawn(
            16, dir.subdir("instances"),
        ).await.unwrap();
        instance_cache.write(
            id.clone(), instance_data.clone(), |_, _| false, true,
        ).await.unwrap();

        // define the mock http client
        let mut http_client= MockConfigInstancesClient::default();
        let result = vec![
            openapi_client::models::BackendConfigInstance {
                id: id.clone(),
                instance: Some(instance_data.clone()),
                target_status: openapi_client::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_REMOVED,
                ..Default::default()
            }
        ];
        let result_cloned = result.clone();
        http_client.set_list_all_config_instances(move || { Ok(result_cloned.clone()) });

        // pull the config instances
        pull_config_instances(
            &metadata_cache,
            &instance_cache,
            &http_client,
            "device_id",
            "token",
        ).await.unwrap();

        // check the metadata cache
        assert_eq!(metadata_cache.size().await.unwrap(), 1);
        let expected = ConfigInstance {
            target_status: TargetStatus::Removed,
            ..existing_instance
        };
        let actual = metadata_cache.read(id.clone()).await.unwrap();
        assert_eq!(expected, actual);

        // check the instance data cache
        assert_eq!(instance_cache.size().await.unwrap(), 1);
        let expected = instance_data;
        let actual = instance_cache.read(id.clone()).await.unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn one_instance_up_to_date() {
        // define the existing instance
        let id = "instance1".to_string();
        let existing_instance = ConfigInstance {
            id: id.clone(),
            target_status: TargetStatus::Deployed,
            ..Default::default()
        };
        let instance_data = json!({
            "instance1": {
                "data": "data1",
                "metadata": "metadata1",
            }
        });

        // define the caches
        let dir = Dir::create_temp_dir("apply").await.unwrap();
        let (metadata_cache, _) = ConfigInstanceCache::spawn(
            16, dir.file("metadata.json"),
        ).await.unwrap();
        metadata_cache.write(
            id.clone(), existing_instance.clone(), |_, _| false, true,
        ).await.unwrap();
        let (instance_cache, _) = ConfigInstanceDataCache::spawn(
            16, dir.subdir("instances"),
        ).await.unwrap();
        instance_cache.write(
            id.clone(), instance_data.clone(), |_, _| false, true,
        ).await.unwrap();

        // define the mock http client
        let mut http_client= MockConfigInstancesClient::default();
        let result = vec![
            openapi_client::models::BackendConfigInstance {
                id: id.clone(),
                instance: Some(instance_data.clone()),
                target_status: openapi_client::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_DEPLOYED,
                ..Default::default()
            }
        ];
        let result_cloned = result.clone();
        http_client.set_list_all_config_instances(move || { Ok(result_cloned.clone()) });

        // pull the config instances
        pull_config_instances(
            &metadata_cache,
            &instance_cache,
            &http_client,
            "device_id",
            "token",
        ).await.unwrap();

        // check the metadata cache
        assert_eq!(metadata_cache.size().await.unwrap(), 1);
        let expected = existing_instance;
        let actual = metadata_cache.read(id.clone()).await.unwrap();
        assert_eq!(expected, actual);

        // check the instance data cache
        assert_eq!(instance_cache.size().await.unwrap(), 1);
        let expected = instance_data;
        let actual = instance_cache.read(id.clone()).await.unwrap();
        assert_eq!(expected, actual);
    }
}