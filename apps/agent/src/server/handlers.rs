use std::sync::Arc;

// internal crates
use crate::authn::token_mngr::TokenManagerExt;
use crate::errors::MiruError;
use crate::models::device::DeviceStatus;
use crate::server::errors::*;
use crate::server::state::ServerState;
use crate::services::config_instances::{get_deployed, get_deployed::GetDeployedArgs};
use crate::services::config_schemas::{hash, hash::HashSchemaArgsI};
use crate::services::device::{get, sync};
use crate::trace;
use crate::utils::version_info;
use openapi_server::models::{
    Error,
    ErrorResponse,
    HealthResponse,
    HashSchemaSerializedRequest,
    HashSerializedConfigSchemaFormat,
    SchemaDigestResponse,
    VersionResponse
};

// external
use axum::{extract::Query, extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;
use serde_json::json;
use tracing::error;

// ================================= AGENT INFO ==================================== //
pub async fn health() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(HealthResponse { status: "ok".to_string() })
    )
}

pub async fn version() -> impl IntoResponse {
    let version_info = version_info();
    (
        StatusCode::OK,
        Json(VersionResponse {
            version: version_info.version,
            commit: version_info.commit,
        })
    )
}


// =============================== CONFIG SCHEMAS ================================== //
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
            ServerErr::AuthnErr(Box::new(ServerAuthnErr {
                source: e,
                trace: trace!(),
            }))
        })?;
        hash::hash_schema(
            &payload,
            &state.caches.cfg_sch_digest,
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

// ============================= CONFIG INSTANCES ================================ //
#[derive(Debug, Deserialize)]
pub struct GetDeployedQueryArgs {
    pub config_type_slug: String,
    pub config_schema_digest: String,
}

pub async fn get_deployed_config_instance(
    Query(query): Query<GetDeployedQueryArgs>,
    State(state): State<Arc<ServerState>>,
) -> impl IntoResponse {
    let service = async move {
        let token = state.token_mngr.get_token().await.map_err(|e| {
            ServerErr::AuthnErr(Box::new(ServerAuthnErr {
                source: e,
                trace: trace!(),
            }))
        })?;

        let device = state.device_file.read().await.map_err(|e| {
            ServerErr::FileSysErr(Box::new(ServerFileSysErr {
                source: e,
                trace: trace!(),
            }))
        })?;

        let args = GetDeployedArgs {
            device_id: device.id.clone(),
            config_type_slug: query.config_type_slug,
            config_schema_digest: query.config_schema_digest,
        };

        get_deployed::get_deployed(
            &args,
            &state.syncer,
            state.caches.cfg_inst.clone(),
            state.caches.cfg_inst_content.clone(),
            &state.caches.cfg_schema,
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
        Ok(cfg_inst) => (StatusCode::OK, Json(json!(cfg_inst))),
        Err(e) => {
            error!("Error reading deployed config instance: {e:?}");
            (e.http_status(), Json(json!(to_error_response(e))))
        }
    }
}

// ================================= DEVICE ======================================== //
pub async fn get_device(
    State(state): State<Arc<ServerState>>,
) -> impl IntoResponse {
    let service = async move {
        let device = get::get_device(&state.device_file).await.map_err(|e| {
            ServerErr::ServiceErr(Box::new(ServerServiceErr {
                source: e,
                trace: trace!(),
            }))
        })?;
        Ok::<openapi_server::models::Device, ServerErr>(
            openapi_server::models::Device {
                object: openapi_server::models::device::Object::Device,
                id: device.id.clone(),
                name: device.name.clone(),
                status: DeviceStatus::to_sdk(&device.status),
                last_synced_at: device.last_synced_at.to_rfc3339(),
                last_connected_at: device.last_connected_at.to_rfc3339(),
                last_disconnected_at: device.last_disconnected_at.to_rfc3339(),
            }
        )
    };

    match service.await {
        Ok(device) => (StatusCode::OK, Json(json!(device))),
        Err(e) => {
            error!("Error getting device: {e:?}");
            (e.http_status(), Json(json!(to_error_response(e))))
        }
    }
}

pub async fn sync_device(
    State(state): State<Arc<ServerState>>,
) -> impl IntoResponse {
    let service = async move {
        sync::sync_device(state.syncer.as_ref()).await.map_err(|e| {
            ServerErr::ServiceErr(Box::new(ServerServiceErr {
                source: e,
                trace: trace!(),
            }))
        })
    };

    match service.await {
        Ok(device) => (StatusCode::OK, Json(json!(device))),
        Err(e) => {
            error!("Error syncing device: {e:?}");
            (e.http_status(), Json(json!(to_error_response(e))))
        }
    }
}

// ================================ UTILITIES ====================================== //
fn to_error_response(e: impl MiruError) -> ErrorResponse {
    ErrorResponse {
        error: Box::new(Error {
            code: e.code().as_str().to_string(),
            params: e.params(),
            message: e.to_string(),
            debug_message: format!("{e:?}"),
        }),
    }
}