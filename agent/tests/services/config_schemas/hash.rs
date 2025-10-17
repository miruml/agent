// internal crates
use std::time::{Duration, Instant};

// internal crates
use miru_agent::crud::prelude::*;
use miru_agent::crypt::sha256;
use miru_agent::errors::MiruError;
use miru_agent::filesys::dir::Dir;
use miru_agent::http::client::HTTPClient;
use miru_agent::http::errors::{HTTPErr, MockErr};
use miru_agent::services::config_schemas::hash;
use miru_agent::storage::digests::{ConfigSchemaDigestCache, ConfigSchemaDigests};
use openapi_client::models::SchemaDigestResponse;
use openapi_server::models::HashSerializedConfigSchemaFormat;

// test crates
use crate::http::mock::MockCfgSchsClient;

// external crates
use serde_json::json;

pub mod errors {
    use super::*;

    #[tokio::test]
    async fn from_server_network_error() {
        let dir = Dir::create_temp_dir("pulled_from_server_resp")
            .await
            .unwrap();
        let (cache, _) = ConfigSchemaDigestCache::spawn(32, dir.file("cache.json"), 1000)
            .await
            .unwrap();

        // create the mock
        let mut mock_client = MockCfgSchsClient::default();
        let server_resp = || -> Result<SchemaDigestResponse, HTTPErr> {
            Err(HTTPErr::MockErr(Box::new(MockErr {
                is_network_connection_error: true,
            })))
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
        let result = hash::hash_schema(&args, &cache, &mock_client, "doesntmatter")
            .await
            .unwrap_err();
        assert!(result.is_network_connection_error());
    }
}

pub mod success {
    use super::*;

    #[tokio::test]
    async fn from_storage() {
        let dir = Dir::create_temp_dir("pulled_from_server_resp")
            .await
            .unwrap();
        let (cache, _) = ConfigSchemaDigestCache::spawn(32, dir.file("cache.json"), 1000)
            .await
            .unwrap();

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
            .write(raw_digest.clone(), digests, |_, _| false, false)
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
        let (cache, _) = ConfigSchemaDigestCache::spawn(32, dir.file("cache.json"), 1000)
            .await
            .unwrap();

        // create the mock
        let mut mock_client = MockCfgSchsClient::default();
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
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), resolved_digest);
        let duration = start.elapsed();
        assert!(duration < Duration::from_millis(20));

        // expect the digest to be cached
        let cached_digest = cache.read(raw_digest).await.unwrap();
        assert_eq!(cached_digest.resolved, resolved_digest);
    }
}
