#[cfg(test)]
mod tests {
    // internal crates

    // internal crates
    use config_agent::filesys::dir::Dir;
    use config_agent::http_client::errors::HTTPErr;
    use config_agent::services::{
        concrete_configs::{
            read_latest,
            read_latest::{ReadLatestArgs, ReadLatestArgsI},
            utils,
        },
        errors::ServiceErr,
    };
    use config_agent::storage::concrete_configs::{ConcreteConfig, ConcreteConfigCache};
    use config_agent::trace;
    use openapi_client::models::BackendConcreteConfig;

    // test crates
    use crate::http_client::mock::MockConcreteConfigsClient;


    pub mod errors {
        use super::*;

        #[tokio::test]
        async fn network_connection_error_and_storage_not_found() {
            let dir = Dir::create_temp_dir("read_latest_errors").await.unwrap();
            let cache = ConcreteConfigCache::spawn(dir);

            // create the mock http client
            let mut http_client = MockConcreteConfigsClient::default();
            http_client.set_read_latest(|| {
                Err(HTTPErr::MockErr {
                    is_network_connection_error: true,
                    trace: trace!(),
                })
            });

            // run the test
            let args = ReadLatestArgs {
                config_slug: "config-slug".to_string(),
                config_schema_digest: "config-schema-digest".to_string(),
            };
            let result = read_latest::read_latest(
                &args,
                &http_client,
                &cache,
            ).await;

            // assert the result
            assert!(matches!(
                result, 
                Err(ServiceErr::LatestConcreteConfigNotFound {
                    config_slug: _,
                    config_schema_digest: _,
                    trace: _,
                }),
            ));
        }

        #[tokio::test]
        async fn non_network_connection_error() {
            let dir = Dir::create_temp_dir("read_latest_errors").await.unwrap();
            let cache = ConcreteConfigCache::spawn(dir);

            // create the mock http client
            let mut http_client = MockConcreteConfigsClient::default();
            http_client.set_read_latest(|| {
                Err(HTTPErr::MockErr {
                    is_network_connection_error: false,
                    trace: trace!(),
                })
            });

            // run the test
            let args = ReadLatestArgs {
                config_slug: "config-slug".to_string(),
                config_schema_digest: "config-schema-digest".to_string(),
            };
            let result = read_latest::read_latest(
                &args,
                &http_client,
                &cache,
            ).await.unwrap_err();

            // assert the result
            assert!(matches!(result, ServiceErr::HTTPErr { .. }));
        }
    }

    pub mod success {
        use super::*;

        #[tokio::test]
        async fn from_storage_network_connection_error() {
            let dir = Dir::create_temp_dir("read_latest_errors").await.unwrap();

            // create the concrete config in storage
            let cache = ConcreteConfigCache::spawn(dir);
            let config_slug = "config-slug";
            let config_schema_digest = "config-schema-digest";
            let concrete_config = ConcreteConfig {
                config_slug: config_slug.to_string(),
                config_schema_digest: config_schema_digest.to_string(),
                ..Default::default()
            };
            cache.write(concrete_config.clone(), false).await.unwrap();

            // create the mock http client
            let mut http_client = MockConcreteConfigsClient::default();
            http_client.set_read_latest(|| {
                Err(HTTPErr::MockErr {
                    is_network_connection_error: true,
                    trace: trace!(),
                })
            });

            // run the test
            let args = ReadLatestArgs {
                config_slug: config_slug.to_string(),
                config_schema_digest: config_schema_digest.to_string(),
            };
            let result = read_latest::read_latest(
                &args,
                &http_client,
                &cache,
            ).await.unwrap();

            let expected = utils::convert_cncr_cfg_storage_to_sdk(concrete_config);
            assert_eq!(result, expected);
        }

        #[tokio::test]
        async fn from_storage_server_not_found() {
            // theoretically this case should never happen. Nonetheless, this is what
            // we would expect to happen if it did.
            let dir = Dir::create_temp_dir("read_latest_errors").await.unwrap();

            // create the concrete config in storage
            let cache = ConcreteConfigCache::spawn(dir);
            let config_slug = "config-slug";
            let config_schema_digest = "config-schema-digest";
            let concrete_config = ConcreteConfig {
                config_slug: config_slug.to_string(),
                config_schema_digest: config_schema_digest.to_string(),
                ..Default::default()
            };
            cache.write(concrete_config.clone(), false).await.unwrap();

            // create the mock http client
            let mut http_client = MockConcreteConfigsClient::default();
            http_client.set_read_latest(move || { Ok(None) });

            // run the test
            let args = ReadLatestArgs {
                config_slug: config_slug.to_string(),
                config_schema_digest: config_schema_digest.to_string(),
            };
            let result = read_latest::read_latest(
                &args,
                &http_client,
                &cache,
            ).await.unwrap();

            let expected = utils::convert_cncr_cfg_storage_to_sdk(concrete_config);
            assert_eq!(result, expected);
        }

        #[tokio::test]
        async fn from_server_found() {
            let dir = Dir::create_temp_dir("read_latest_errors").await.unwrap();
            let cache = ConcreteConfigCache::spawn(dir);

            // create the mock http client
            let backend_concrete_config = BackendConcreteConfig::default();
            let backend_concrete_config_clone = backend_concrete_config.clone();
            let mut http_client = MockConcreteConfigsClient::default();
            http_client.set_read_latest(
                move || { Ok(Some(backend_concrete_config_clone.clone())) }
            );

            // run the test
            let args = ReadLatestArgs {
                config_slug: "config-slug".to_string(),
                config_schema_digest: "config-schema-digest".to_string(),
            };
            let result = read_latest::read_latest(
                &args,
                &http_client,
                &cache,
            ).await.unwrap();

            let storage_concrete_config = utils::convert_cncr_cfg_backend_to_storage(
                backend_concrete_config,
                args.config_slug().to_string(),
                args.config_schema_digest().to_string(),
            );
            let expected = utils::convert_cncr_cfg_storage_to_sdk(
                storage_concrete_config.clone(),
            );
            assert_eq!(result, expected);

            // cache should have been updated
            let cached_concrete_config = cache.read(
                args.config_slug(),
                args.config_schema_digest(),
            ).await.unwrap();
            assert_eq!(cached_concrete_config, storage_concrete_config);
        }
    }
}