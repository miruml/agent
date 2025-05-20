// internal crates
use std::time::{Duration, Instant};

// internal crates
use config_agent::crypt::sha256;
use config_agent::filesys::dir::Dir;
use config_agent::http::client::HTTPClient;
use config_agent::http::errors::{HTTPErr, MockErr};
use config_agent::services::config_schemas::hash;
use config_agent::services::errors::{ServiceErr, ServiceHTTPErr};
use config_agent::storage::digests::{ConfigSchemaDigestCache, ConfigSchemaDigests};
use config_agent::trace;
use openapi_client::models::SchemaDigestResponse;
use openapi_server::models::HashSerializedConfigSchemaFormat;

// test crates
use crate::http::mock::MockConfigSchemasClient;

// external crates
use serde_json::json;

pub mod errors {
    use super::*;

    #[tokio::test]
    async fn from_server_network_error() {
        let dir = Dir::create_temp_dir("pulled_from_server_resp")
            .await
            .unwrap();
        let (cache, _) = ConfigSchemaDigestCache::spawn(dir);

        // create the mock
        let mut mock_client = MockConfigSchemasClient::default();
        let server_resp = || -> Result<SchemaDigestResponse, HTTPErr> {
            Err(HTTPErr::MockErr(MockErr {
                is_network_connection_error: true,
                trace: trace!(),
            }))
        };
        mock_client.set_hash_schema(server_resp);

        // create the schema
        let schema = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            }
        })
        .to_string();
        let args = hash::HashSchemaArgs {
            schema: schema.as_bytes().to_vec(),
            format: HashSerializedConfigSchemaFormat::HASH_SERIALIZED_CONFIG_SCHEMA_FORMAT_JSON,
        };

        // run the test
        let result = hash::hash_schema(&args, &cache, &mock_client, "doesntmatter").await;
        assert!(matches!(
            result,
            Err(ServiceErr::HTTPErr(ServiceHTTPErr {
                source: HTTPErr::MockErr(MockErr {
                    is_network_connection_error: true,
                    trace: _,
                }),
                trace: _,
            }))
        ));
    }
}

pub mod success {
    use config_agent::filesys::dir::Dir;

    use super::*;

    #[tokio::test]
    async fn from_storage() {
        let dir = Dir::create_temp_dir("pulled_from_server_resp")
            .await
            .unwrap();
        let (cache, _) = ConfigSchemaDigestCache::spawn(dir);

        // define the schema
        let schema = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            }
        })
        .to_string();
        let raw_digest = sha256::hash_str(&schema);
        let resolved_digest = "resolved_digest";
        let args = hash::HashSchemaArgs {
            schema: schema.as_bytes().to_vec(),
            format: HashSerializedConfigSchemaFormat::HASH_SERIALIZED_CONFIG_SCHEMA_FORMAT_JSON,
        };

        // save the digest to the storage
        let digests = ConfigSchemaDigests {
            raw: raw_digest.clone(),
            resolved: resolved_digest.to_string(),
        };
        cache
            .write(raw_digest.clone(), digests, false)
            .await
            .unwrap();

        let http_client = HTTPClient::new("doesntmatter").await;

        // first access should be less than 100ms
        let start = Instant::now();
        let result = hash::hash_schema(&args, &cache, &http_client, "doesntmatter").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), resolved_digest);
        let duration = start.elapsed();
        assert!(duration < Duration::from_millis(100));

        // second access should be less than 1ms
        let start = Instant::now();
        let result = hash::hash_schema(&args, &cache, &http_client, "doesntmatter").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), resolved_digest);
        let duration = start.elapsed();
        assert!(duration < Duration::from_millis(100));
    }

    #[tokio::test]
    async fn from_server() {
        let dir = Dir::create_temp_dir("pulled_from_server_resp")
            .await
            .unwrap();
        let (cache, _) = ConfigSchemaDigestCache::spawn(dir);

        // create the mock
        let mut mock_client = MockConfigSchemasClient::default();
        let resolved_digest = "sha256:a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r";
        let server_resp = || -> Result<SchemaDigestResponse, HTTPErr> {
            Ok(SchemaDigestResponse {
                digest: resolved_digest.to_string(),
            })
        };
        mock_client.set_hash_schema(server_resp);

        // create the schema
        let schema = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            }
        })
        .to_string();
        let raw_digest = sha256::hash_str(&schema);
        let args = hash::HashSchemaArgs {
            schema: schema.as_bytes().to_vec(),
            format: HashSerializedConfigSchemaFormat::HASH_SERIALIZED_CONFIG_SCHEMA_FORMAT_JSON,
        };

        // run the test
        let start = Instant::now();
        let result = hash::hash_schema(&args, &cache, &mock_client, "doesntmatter").await;
        println!("result: {:?}", result);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), resolved_digest);
        let duration = start.elapsed();
        assert!(duration < Duration::from_millis(20));

        // expect the digest to be cached
        let cached_digest = cache.read(raw_digest).await.unwrap();
        assert_eq!(cached_digest.resolved, resolved_digest);
    }
}
