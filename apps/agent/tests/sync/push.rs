// internal crates
use crate::http::mock::HistoryConfigInstancesClient;
use config_agent::filesys::dir::Dir;
use config_agent::models::config_instance::{ActivityStatus, ConfigInstance, ErrorStatus};
use config_agent::storage::config_instances::ConfigInstanceCache;
use config_agent::sync::push::push_config_instances;
use openapi_client::models::UpdateConfigInstanceRequest;

pub mod push_config_instances_func {
    use super::*;

    #[tokio::test]
    async fn no_instances() {
        // define the caches
        let dir = Dir::create_temp_dir("apply").await.unwrap();
        let (metadata_cache, _) = ConfigInstanceCache::spawn(16, dir.file("metadata.json"), 1000)
            .await
            .unwrap();

        // define the mock http client
        let http_client = HistoryConfigInstancesClient::default();

        // pull the config instances
        push_config_instances(&metadata_cache, &http_client, "token")
            .await
            .unwrap();

        // check the history
        let history = http_client.get_update_config_instance_requests();
        assert_eq!(history.len(), 0);
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
        let http_client = HistoryConfigInstancesClient::default();

        // pull the config instances
        push_config_instances(&metadata_cache, &http_client, "token")
            .await
            .unwrap();

        // check the history
        let history = http_client.get_update_config_instance_requests();
        assert_eq!(history.len(), 0);
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
        let http_client = HistoryConfigInstancesClient::default();

        // pull the config instances
        push_config_instances(&metadata_cache, &http_client, "token")
            .await
            .unwrap();

        // check the history
        let history = http_client.get_update_config_instance_requests();
        assert_eq!(history.len(), 1);
        let actual = history[0].clone();
        let expected = UpdateConfigInstanceRequest {
            activity_status: Some(openapi_client::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_DEPLOYED),
            error_status: Some(openapi_client::models::ConfigInstanceErrorStatus::CONFIG_INSTANCE_ERROR_STATUS_RETRYING),
        };
        assert_eq!(actual, expected);
    }
}
