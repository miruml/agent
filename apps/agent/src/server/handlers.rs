use std::sync::Arc;

// internal crates
use crate::errors::MiruError;
use crate::server::errors::{ServerAuthErr, ServerErr, ServerServiceErr};
use crate::server::state::ServerState;
use crate::services::config_instances::{
    read_deployed,
    read_deployed::ReadDeployedArgs,
};
use crate::services::config_schemas::{hash, hash::HashSchemaArgsI};
use crate::trace;
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
            ServerErr::AuthErr(Box::new(ServerAuthErr {
                source: e,
                trace: trace!(),
            }))
        })?;
        hash::hash_schema(
            &payload,
            &state.cfg_sch_digest_cache,
            &state.http_client,
            &token.token,
        )
        .await
        .map_err(|e| {
            ServerErr::ServiceErr(Box::new(ServerServiceErr {
                source: e,
                trace: trace!(),
            }))
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
pub struct ReadDeployedQueryArgs {
    pub config_type_slug: String,
    pub config_schema_digest: String,
}

pub async fn read_deployed_config_instance(
    Query(query): Query<ReadDeployedQueryArgs>,
    State(state): State<Arc<ServerState>>,
) -> impl IntoResponse {
    let service = async move {
        let token = state.token_mngr.get_token().await.map_err(|e| {
            ServerErr::AuthErr(Box::new(ServerAuthErr {
                source: e,
                trace: trace!(),
            }))
        })?;

        let args = ReadDeployedArgs {
            device_id: state.device_id.clone(),
            config_type_slug: query.config_type_slug,
            config_schema_digest: query.config_schema_digest,
        };

        read_deployed::read_deployed(
            &args,
            &state.syncer,
            state.cfg_inst_metadata_cache.clone(),
            state.cfg_inst_data_cache.clone(),
            &state.cfg_schema_cache,
            &state.http_client,
            &token.token,
        )
        .await
        .map_err(|e| {
            ServerErr::ServiceErr(Box::new(ServerServiceErr {
                source: e,
                trace: trace!(),
            }))
        })
    };

    match service.await {
        Ok(config_instance) => (StatusCode::OK, Json(json!(config_instance))),
        Err(e) => {
            error!("Error reading deployed config instance: {:?}", e);
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
