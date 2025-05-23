use std::sync::Arc;

// internal crates
use crate::errors::MiruError;
use crate::server::errors::{ServerAuthErr, ServerErr, ServerServiceErr};
use crate::server::state::ServerState;
use crate::services::concrete_configs::{
    read_latest, read_latest::ReadLatestArgs, refresh_latest, refresh_latest::RefreshLatestArgs,
};
use crate::services::config_schemas::{hash, hash::HashSchemaArgsI};
use crate::trace;
use openapi_server::models::RefreshLatestConcreteConfigRequest;
use openapi_server::models::SchemaDigestResponse;
use openapi_server::models::{Error, ErrorResponse};
use openapi_server::models::{HashSchemaSerializedRequest, HashSerializedConfigSchemaFormat};

// external
use axum::{extract::Query, extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;
use serde_json::json;
use tracing::error;

impl HashSchemaArgsI for HashSchemaSerializedRequest {
    fn schema(&self) -> &Vec<u8> {
        &self.schema
    }
    fn format(&self) -> &HashSerializedConfigSchemaFormat {
        &self.format
    }
}

pub async fn hash_schema(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<HashSchemaSerializedRequest>,
) -> impl IntoResponse {
    let service = async move {
        let token = state.token_mngr.get_token().await.map_err(|e| {
            ServerErr::AuthErr(ServerAuthErr {
                source: Box::new(e),
                trace: trace!(),
            })
        })?;
        hash::hash_schema(
            &payload,
            &state.config_schema_digest_cache,
            &state.http_client,
            &token.token,
        )
        .await
        .map_err(|e| {
            ServerErr::ServiceErr(ServerServiceErr {
                source: Box::new(e),
                trace: trace!(),
            })
        })
    };

    match service.await {
        Ok(digest) => (StatusCode::OK, Json(json!(SchemaDigestResponse { digest }))),
        Err(e) => {
            error!("Error hashing schema: {:?}", e);
            (e.http_status(), Json(json!(to_error_response(e))))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ReadLatestQueryArgs {
    pub config_slug: String,
    pub config_schema_digest: String,
}

pub async fn read_latest_concrete_config(
    Query(query): Query<ReadLatestQueryArgs>,
    State(state): State<Arc<ServerState>>,
) -> impl IntoResponse {
    let service = async move {
        let token = state.token_mngr.get_token().await.map_err(|e| {
            ServerErr::AuthErr(ServerAuthErr {
                source: Box::new(e),
                trace: trace!(),
            })
        })?;

        let args = ReadLatestArgs {
            client_id: state.client_id.clone(),
            config_slug: query.config_slug,
            config_schema_digest: query.config_schema_digest,
        };

        read_latest::read_latest(
            &args,
            &state.concrete_config_cache,
            &state.http_client,
            &token.token,
        )
        .await
        .map_err(|e| {
            ServerErr::ServiceErr(ServerServiceErr {
                source: Box::new(e),
                trace: trace!(),
            })
        })
    };

    match service.await {
        Ok(concrete_config) => (StatusCode::OK, Json(json!(concrete_config))),
        Err(e) => {
            error!("Error reading latest concrete config: {:?}", e);
            (e.http_status(), Json(json!(to_error_response(e))))
        }
    }
}

pub async fn refresh_latest_concrete_config(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<RefreshLatestConcreteConfigRequest>,
) -> impl IntoResponse {
    let service = async move {
        let token = state.token_mngr.get_token().await.map_err(|e| {
            ServerErr::AuthErr(ServerAuthErr {
                source: Box::new(e),
                trace: trace!(),
            })
        })?;

        let args = RefreshLatestArgs {
            client_id: state.client_id.clone(),
            config_slug: payload.config_slug,
            config_schema_digest: payload.config_schema_digest,
        };

        refresh_latest::refresh_latest(
            &args,
            &state.concrete_config_cache,
            &state.http_client,
            &token.token,
        )
        .await
        .map_err(|e| {
            ServerErr::ServiceErr(ServerServiceErr {
                source: Box::new(e),
                trace: trace!(),
            })
        })
    };

    match service.await {
        Ok(concrete_config) => (StatusCode::OK, Json(json!(concrete_config))),
        Err(e) => {
            error!("Error refreshing latest concrete config: {:?}", e);
            (e.http_status(), Json(json!(to_error_response(e))))
        }
    }
}

fn to_error_response(e: impl MiruError) -> ErrorResponse {
    ErrorResponse {
        error: Box::new(Error {
            code: e.code().as_str().to_string(),
            params: e.params(),
            message: e.to_string(),
            debug_message: format!("{:?}", e),
        }),
    }
}
