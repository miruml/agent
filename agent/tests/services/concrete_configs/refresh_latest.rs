// internal crates
use config_agent::filesys::dir::Dir;
use config_agent::http::errors::{HTTPErr, MockErr};
use config_agent::services::{
    concrete_configs::{
        refresh_latest,
        refresh_latest::{RefreshLatestArgs, RefreshLatestArgsI},
        utils,
    },
    errors::ServiceErr,
};
use config_agent::storage::concrete_configs::{ConcreteConfigCache, ConcreteConfigCacheKey};
use config_agent::trace;
use openapi_client::models::BackendConcreteConfig;

// test crates
use crate::http::mock::MockConcreteConfigsClient;

pub mod errors {
    use super::*;

    #[tokio::test]
    async fn server_request_error() {
        let dir = Dir::create_temp_dir("refresh_latest_errors").await.unwrap();
        let (cache, _) = ConcreteConfigCache::spawn(dir);

        // create the mock http client
        let mut http_client = MockConcreteConfigsClient::default();
        http_client.set_refresh_latest(|| {
            Err(HTTPErr::MockErr(MockErr {
                is_network_connection_error: true,
                trace: trace!(),
            }))
        });

        // run the test
        let args = RefreshLatestArgs {
            config_slug: "config-slug".to_string(),
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
        let (cache, _) = ConcreteConfigCache::spawn(dir);

        // create the mock http client
        let backend_concrete_config = BackendConcreteConfig::default();
        let backend_concrete_config_clone = backend_concrete_config.clone();
        let mut http_client = MockConcreteConfigsClient::default();
        http_client.set_refresh_latest(move || Ok(backend_concrete_config_clone.clone()));

        // run the test
        let args = RefreshLatestArgs {
            config_slug: "config-slug".to_string(),
            config_schema_digest: "config-schema-digest".to_string(),
        };
        let result = refresh_latest::refresh_latest(&args, &cache, &http_client, "doesntmatter")
            .await
            .unwrap();

        let storage_concrete_config = utils::convert_cncr_cfg_backend_to_storage(
            backend_concrete_config,
            args.config_slug().to_string(),
            args.config_schema_digest().to_string(),
        );
        let expected = utils::convert_cncr_cfg_storage_to_sdk(storage_concrete_config.clone());
        assert_eq!(result, expected);

        // cache should have been updated
        let key = ConcreteConfigCacheKey {
            config_slug: args.config_slug().to_string(),
            config_schema_digest: args.config_schema_digest().to_string(),
        };
        let cached_concrete_config = cache.read(key).await.unwrap();
        assert_eq!(cached_concrete_config, storage_concrete_config);
    }
}
