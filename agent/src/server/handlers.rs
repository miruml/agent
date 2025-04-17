use std::sync::Arc;

// internal crates
use crate::http_client::client::HTTPClient;
use crate::services::config_schemas::hash;
use crate::storage::digests::AsyncConfigSchemaDigestCache;
use openapi_server::models::SchemaDigestResponse;
use openapi_server::models::HashSchemaRequest;

// external
use axum::{
    http::StatusCode,
    Json,
};
use serde_json::json;
use tracing::error;

pub async fn hash_schema(
    Json(payload): Json<HashSchemaRequest>,
    http_client: Arc<HTTPClient>,
    cache: Arc<AsyncConfigSchemaDigestCache>,
) -> (StatusCode, Json<serde_json::Value>) {
    let digest = hash::hash_schema(
        &payload.schema,
        &http_client,
        &cache,
    ).await;

    match digest {
        Ok(digest) => (StatusCode::OK, Json(json!(SchemaDigestResponse { digest }))),
        Err(e) => {
            error!("Error hashing schema: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Internal server error" })))
        }
    }
}

// async fn refresh_latest_concrete_config(
//     payload: Query<RenderLatestConcreteConfigRequest>,
// ) -> (StatusCode, Json<serde_json::Value>) {
//     (StatusCode::OK, Json(json!({
//         "object": "concrete_config",
//         "id": "cncr_cfg_123",
//         "created_at": "2021-01-01T00:00:00Z",
//         "created_by_id": "usr_123",
//         "client_id": "cli_123",
//         "config_schema_id": "cfg_sch_123",
//         "concrete_config": {
//             "device_id": "device_23jt0321p9123434gsdf",
//             "speed": 100
//         }
//     })))
// }

// async fn read_latest_concrete_config(
//     Query(payload): Query<RenderLatestConcreteConfigRequest>,
// ) -> (StatusCode, Json<serde_json::Value>) {
//     (StatusCode::OK, Json(json!({
//         "object": "concrete_config",
//         "id": "cncr_cfg_123",
//         "created_at": "2021-01-01T00:00:00Z",
//         "created_by_id": "usr_123",
//         "client_id": "cli_123",
//         "config_schema_id": "cfg_sch_123",
//         "concrete_config": {
//             "device_id": "device_23jt0321p9123434gsdf",
//             "speed": 100
//         }
//     })))
// }