use std::sync::Arc;

// internal crates
use crate::errors::MiruError;
use crate::server::api::AppState;
use crate::services::concrete_configs::{
    read_latest, read_latest::ReadLatestArgs, refresh_latest, refresh_latest::RefreshLatestArgsI,
};
use crate::services::config_schemas::{hash, hash::HashSchemaArgsI};
use openapi_server::models::{
    HashSchemaSerializedRequest, HashSerializedConfigSchemaFormat,
};
use openapi_server::models::RefreshLatestConcreteConfigRequest;
use openapi_server::models::SchemaDigestResponse;
use openapi_server::models::{Error, ErrorResponse};

// external
use axum::{extract::Query, extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use tracing::error;

impl HashSchemaArgsI for HashSchemaSerializedRequest {
    fn schema(&self) -> &str {
        &self.schema
    }
    fn format(&self) -> &HashSerializedConfigSchemaFormat {
        &self.format
    }
}

pub async fn hash_schema(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<HashSchemaSerializedRequest>,
) -> impl IntoResponse {
    let result = hash::hash_schema(
        &payload,
        &state.http_client,
        &state.config_schema_digest_cache,
    )
    .await;

    match result {
        Ok(digest) => (StatusCode::OK, Json(json!(SchemaDigestResponse { digest }))),
        Err(e) => {
            error!("Error hashing schema: {:?}", e);
            (
                e.http_status(),
                Json(json!(to_error_response(e))),
            )
        }
    }
}

pub async fn read_latest_concrete_config(
    Query(query): Query<ReadLatestArgs>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let result =
        read_latest::read_latest(&query, &state.http_client, &state.concrete_config_cache).await;

    match result {
        Ok(concrete_config) => (StatusCode::OK, Json(json!(concrete_config))),
        Err(e) => {
            error!("Error reading latest concrete config: {:?}", e);
            (
                e.http_status(),
                Json(json!(to_error_response(e))),
            )
        }
    }
}

impl RefreshLatestArgsI for RefreshLatestConcreteConfigRequest {
    fn config_slug(&self) -> &str {
        &self.config_slug
    }
    fn config_schema_digest(&self) -> &str {
        &self.config_schema_digest
    }
}

pub async fn refresh_latest_concrete_config(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RefreshLatestConcreteConfigRequest>,
) -> impl IntoResponse {
    let result =
        refresh_latest::refresh_latest(&payload, &state.http_client, &state.concrete_config_cache)
            .await;

    match result {
        Ok(concrete_config) => (StatusCode::OK, Json(json!(concrete_config))),
        Err(e) => {
            error!("Error refreshing latest concrete config: {:?}", e);
            (
                e.http_status(),
                Json(json!(to_error_response(e))),
            )
        }
    }
}

fn to_error_response(e: impl MiruError) -> ErrorResponse {
    ErrorResponse {
        error: Box::new(Error {
            code: e.code().to_string().to_string(),
            params: e.params(),
            message: e.to_string(),
            debug_message: format!("{:?}", e),
        }),
    }
}