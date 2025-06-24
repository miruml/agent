// internal crates
use config_agent::filesys::dir::Dir;
use config_agent::http::errors::{HTTPErr, MockErr};
use config_agent::models::config_instance::{
    ConfigInstance,
    convert_cfg_inst_backend_to_storage,
    convert_cfg_inst_storage_to_sdk,
};
use config_agent::services::{
    config_instances::{
        read_latest,
        read_latest::{ReadLatestArgs, ReadLatestArgsI},
    },
    errors::{LatestConfigInstanceNotFound, ServiceErr},
};
use config_agent::storage::config_instances::{
    ConfigInstanceCache, ConfigInstanceCacheKey,
};
use config_agent::trace;
use openapi_client::models::BackendConfigInstance;

// test crates
use crate::http::mock::MockConfigInstancesClient;

pub mod errors {
    use super::*;

    #[tokio::test]
    async fn network_connection_error_and_storage_not_found() {
        let dir = Dir::create_temp_dir("read_latest_errors").await.unwrap();
        let (cache, _) = ConfigInstanceCache::spawn(dir);

        // create the mock http client
        let mut http_client = MockConfigInstancesClient::default();
        http_client.set_read_latest(|| {
            Err(HTTPErr::MockErr(MockErr {
                is_network_connection_error: true,
                trace: trace!(),
            }))
        });

        // run the test
        let args = ReadLatestArgs {
            device_id: "device-id".to_string(),
            config_type_slug: "config-type-slug".to_string(),
            config_schema_digest: "config-schema-digest".to_string(),
        };
        let result = read_latest::read_latest(&args, &cache, &http_client, "doesntmatter").await;

        // assert the result
        assert!(matches!(
            result,
            Err(ServiceErr::LatestConfigInstanceNotFound(
                LatestConfigInstanceNotFound {
                    config_type_slug: _,
                    config_schema_digest: _,
                    trace: _,
                }
            )),
        ));
    }

    #[tokio::test]
    async fn non_network_connection_error() {
        let dir = Dir::create_temp_dir("read_latest_errors").await.unwrap();
        let (cache, _) = ConfigInstanceCache::spawn(dir);

        // create the mock http client
        let mut http_client = MockConfigInstancesClient::default();
        http_client.set_read_latest(|| {
            Err(HTTPErr::MockErr(MockErr {
                is_network_connection_error: false,
                trace: trace!(),
            }))
        });

        // run the test
        let args = ReadLatestArgs {
            device_id: "device-id".to_string(),
            config_type_slug: "config-type-slug".to_string(),
            config_schema_digest: "config-schema-digest".to_string(),
        };
        let result = read_latest::read_latest(&args, &cache, &http_client, "doesntmatter")
            .await
            .unwrap_err();

        // assert the result
        assert!(matches!(result, ServiceErr::HTTPErr { .. }));
    }
}

pub mod success {
    use super::*;

    #[tokio::test]
    async fn from_storage_network_connection_error() {
        let dir = Dir::create_temp_dir("read_latest_errors").await.unwrap();

        // create the config instance in storage
        let (cache, _) = ConfigInstanceCache::spawn(dir);
        let config_type_slug = "config-type-slug";
        let config_schema_digest = "config-schema-digest";
        let config_instance = ConfigInstance {
            config_type_slug: config_type_slug.to_string(),
            config_schema_digest: config_schema_digest.to_string(),
            ..Default::default()
        };
        let key = ConfigInstanceCacheKey {
            config_type_slug: config_type_slug.to_string(),
            config_schema_digest: config_schema_digest.to_string(),
        };
        cache
            .write(key, config_instance.clone(), false)
            .await
            .unwrap();

        // create the mock http client
        let mut http_client = MockConfigInstancesClient::default();
        http_client.set_read_latest(|| {
            Err(HTTPErr::MockErr(MockErr {
                is_network_connection_error: true,
                trace: trace!(),
            }))
        });

        // run the test
        let args = ReadLatestArgs {
            device_id: "device-id".to_string(),
            config_type_slug: config_type_slug.to_string(),
            config_schema_digest: config_schema_digest.to_string(),
        };
        let result = read_latest::read_latest(&args, &cache, &http_client, "doesntmatter")
            .await
            .unwrap();

        let expected = convert_cfg_inst_storage_to_sdk(config_instance);
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn from_storage_server_not_found() {
        // theoretically this case should never happen. Nonetheless, this is what
        // we would expect to happen if it did.
        let dir = Dir::create_temp_dir("read_latest_errors").await.unwrap();

        // create the config instance in storage
        let (cache, _) = ConfigInstanceCache::spawn(dir);
        let config_type_slug = "config-type-slug";
        let config_schema_digest = "config-schema-digest";
        let config_instance = ConfigInstance {
            config_type_slug: config_type_slug.to_string(),
            config_schema_digest: config_schema_digest.to_string(),
            ..Default::default()
        };
        let key = ConfigInstanceCacheKey {
            config_type_slug: config_type_slug.to_string(),
            config_schema_digest: config_schema_digest.to_string(),
        };
        cache
            .write(key, config_instance.clone(), false)
            .await
            .unwrap();

        // create the mock http client
        let mut http_client = MockConfigInstancesClient::default();
        http_client.set_read_latest(move || Ok(None));

        // run the test
        let args = ReadLatestArgs {
            device_id: "device-id".to_string(),
            config_type_slug: config_type_slug.to_string(),
            config_schema_digest: config_schema_digest.to_string(),
        };
        let result = read_latest::read_latest(&args, &cache, &http_client, "doesntmatter")
            .await
            .unwrap();

        let expected = convert_cfg_inst_storage_to_sdk(config_instance);
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn from_server_found() {
        let dir = Dir::create_temp_dir("read_latest_errors").await.unwrap();
        let (cache, _) = ConfigInstanceCache::spawn(dir);

        // create the mock http client
        let backend_config_instance = BackendConfigInstance::default();
        let backend_config_instance_clone = backend_config_instance.clone();
        let mut http_client = MockConfigInstancesClient::default();
        http_client.set_read_latest(move || Ok(Some(backend_config_instance_clone.clone())));

        // run the test
        let args = ReadLatestArgs {
            device_id: "device-id".to_string(),
            config_type_slug: "config-type-slug".to_string(),
            config_schema_digest: "config-schema-digest".to_string(),
        };
        let result = read_latest::read_latest(&args, &cache, &http_client, "doesntmatter")
            .await
            .unwrap();

        let storage_config_instance = convert_cfg_inst_backend_to_storage(
            backend_config_instance,
            args.config_type_slug().to_string(),
            args.config_schema_digest().to_string(),
        );
        let expected = convert_cfg_inst_storage_to_sdk(storage_config_instance.clone());
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
