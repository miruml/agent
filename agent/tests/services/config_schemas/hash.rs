#[cfg(test)]
mod tests {
    // internal crates
    use std::time::{Duration, Instant};

    // internal crates
    use config_agent::errors::MiruError;
    use config_agent::http_client::client::HTTPClient;
    use config_agent::services::config_schemas::hash;
    use config_agent::storage::{
        digests::{ConfigSchemaDigestCache, ConfigSchemaDigests},
        layout::StorageLayout,
    };
    use config_agent::utils;
    use openapi_client::models::SchemaDigestResponse;

    // test crates
    use crate::http_client::mock::MockConfigSchemasSuccess;

    // external crates
    use serde_json::json;
    #[allow(unused_imports)]
    use tracing::{debug, error, info, trace, warn};

#[cfg(test)]
pub mod errors {
    use super::*;



    #[test]
    fn invalid_schema() {
        assert!(false);
    }
}

pub mod success {
    use config_agent::filesys::dir::Dir;

    use super::*;

    #[tokio::test]
    async fn from_storage() {
        let dir = Dir::create_temp_dir("pulled_from_server_resp").await.unwrap();
        let cache = ConfigSchemaDigestCache::spawn(dir);

        // define the schema
        let schema = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            }
        });
        let raw_digest = utils::hash_json(&schema);
        let resolved_digest = "resolved_digest";

        // save the digest to the storage
        let digests = ConfigSchemaDigests {
            raw: raw_digest.clone(),
            resolved: resolved_digest.to_string(),
        };
        cache.write(digests, false).await.unwrap();


        let http_client = HTTPClient::new().await;

        // first access should be less than 100ms
        let start = Instant::now();
        let result = hash::hash_schema(
            &schema,
            &http_client,
            &cache,
        ).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), resolved_digest);
        let duration = start.elapsed();
        assert!(duration < Duration::from_millis(100));

        // second access should be less than 1ms
        let start = Instant::now();
        let result = hash::hash_schema(
            &schema,
            &http_client,
            &cache,
        ).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), resolved_digest);
        let duration = start.elapsed();
        assert!(duration < Duration::from_millis(100));
    }

    #[tokio::test]
    async fn from_server() {
        let dir = Dir::create_temp_dir("pulled_from_server_resp").await.unwrap();
        let cache = ConfigSchemaDigestCache::spawn(dir);

        // create the mock
        let mut mock_client = MockConfigSchemasSuccess::default();
        let resolved_digest = "sha256:a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r";
        let server_resp = SchemaDigestResponse {
            digest: resolved_digest.to_string(),
        };
        mock_client.set_hash_schema_result(server_resp);

        // create the schema
        let schema = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            }
        });

        // run the test
        let start = Instant::now();
        let result = hash::hash_schema(
            &schema,
            &mock_client,
            &cache,
        ).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), resolved_digest);
        let duration = start.elapsed();
        assert!(duration < Duration::from_millis(1));

        // expect the digest to be cached
        let raw_digest = utils::hash_json(&schema);
        let cached_digest = cache.read(&raw_digest).await.unwrap();
        assert_eq!(cached_digest.resolved, resolved_digest);
    }
}
}