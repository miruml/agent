// internal crates
use config_agent::filesys::dir::Dir;
use config_agent::http::errors::{HTTPErr, MockErr};
use config_agent::services::{
    config_instances::{
        refresh_latest,
        refresh_latest::{RefreshLatestArgs, RefreshLatestArgsI},
        utils,
    },
    errors::ServiceErr,
};
use config_agent::storage::config_instances::{ConfigInstanceCache, ConfigInstanceCacheKey};
use config_agent::trace;
use openapi_client::models::BackendConfigInstance;

// test crates
use crate::http::mock::MockConfigInstancesClient;

pub mod errors {
    use super::*;

    #[tokio::test]
    async fn server_request_error() {
        let dir = Dir::create_temp_dir("refresh_latest_errors").await.unwrap();
        let (cache, _) = ConfigInstanceCache::spawn(dir);

        // create the mock http client
        let mut http_client = MockConfigInstancesClient::default();
        http_client.set_refresh_latest(|| {
            Err(HTTPErr::MockErr(MockErr {
                is_network_connection_error: true,
                trace: trace!(),
            }))
        });

        // run the test
        let args = RefreshLatestArgs {
            device_id: "device-id".to_string(),
            config_type_slug: "config-type-slug".to_string(),
            config_schema_digest: "config-schema-digest".to_string(),
        };
        let result = refresh_latest::refresh_latest(&args, &cache, &http_client, "doesntmatter")
            .await
            .unwrap_err();

        // assert the result
        assert!(matches!(result, ServiceErr::HTTPErr { .. }));
    }
}

pub mod success {
    use super::*;

    #[tokio::test]
    async fn from_server() {
        let dir = Dir::create_temp_dir("refresh_latest_success")
            .await
            .unwrap();
        let (cache, _) = ConfigInstanceCache::spawn(dir);

        // create the mock http client
        let backend_config_instance = BackendConfigInstance::default();
        let backend_config_instance_clone = backend_config_instance.clone();
        let mut http_client = MockConfigInstancesClient::default();
        http_client.set_refresh_latest(move || Ok(backend_config_instance_clone.clone()));

        // run the test
        let args = RefreshLatestArgs {
            device_id: "device-id".to_string(),
            config_type_slug: "config-type-slug".to_string(),
            config_schema_digest: "config-schema-digest".to_string(),
        };
        let result = refresh_latest::refresh_latest(&args, &cache, &http_client, "doesntmatter")
            .await
            .unwrap();

        let storage_config_instance = utils::convert_cfg_inst_backend_to_storage(
            backend_config_instance,
            args.config_type_slug().to_string(),
            args.config_schema_digest().to_string(),
        );
        let expected = utils::convert_cfg_inst_storage_to_sdk(storage_config_instance.clone());
        assert_eq!(result, expected);

        // cache should have been updated
        let key = ConfigInstanceCacheKey {
            config_type_slug: args.config_type_slug().to_string(),
            config_schema_digest: args.config_schema_digest().to_string(),
        };
        let cached_config_instance = cache.read(key).await.unwrap();
        assert_eq!(cached_config_instance, storage_config_instance);
    }
}
