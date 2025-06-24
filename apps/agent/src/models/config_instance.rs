// external crates
use serde::Deserialize;
use serde::Serialize;

// =============================== TARGET STATUS ================================== //
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum ConfigInstanceTargetStatus {
    #[default]
    #[serde(rename = "created")]
    Created,
    #[serde(rename = "deployed")]
    Deployed,
    #[serde(rename = "removed")]
    Removed,
}

pub fn convert_target_status_backend_to_storage(
    target_status: openapi_client::models::ConfigInstanceTargetStatus,
) -> ConfigInstanceTargetStatus {
    match target_status {
        openapi_client::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_CREATED => ConfigInstanceTargetStatus::Created,
        openapi_client::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_DEPLOYED => ConfigInstanceTargetStatus::Deployed,
        openapi_client::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_REMOVED => ConfigInstanceTargetStatus::Removed,
    }
}

pub fn convert_target_status_storage_to_sdk(
    target_status: ConfigInstanceTargetStatus,
) -> openapi_server::models::ConfigInstanceTargetStatus {
    match target_status {
        ConfigInstanceTargetStatus::Created => openapi_server::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_CREATED,
        ConfigInstanceTargetStatus::Deployed => openapi_server::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_DEPLOYED,
        ConfigInstanceTargetStatus::Removed => openapi_server::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_REMOVED,
    }
}

// ================================== STATUS ======================================= //
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum ConfigInstanceStatus {
    #[default]
    #[serde(rename = "created")]
    Created,
    #[serde(rename = "queued")]
    Queued,
    #[serde(rename = "deployed")]
    Deployed,
    #[serde(rename = "removed")]
    Removed,
    #[serde(rename = "failed")]
    Failed,
}

pub fn convert_status_backend_to_storage(
    status: openapi_client::models::ConfigInstanceStatus,
) -> ConfigInstanceStatus {
    match status {
        openapi_client::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_CREATED => {
            ConfigInstanceStatus::Created
        }
        openapi_client::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_QUEUED => {
            ConfigInstanceStatus::Queued
        }
        openapi_client::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_DEPLOYED => {
            ConfigInstanceStatus::Deployed
        }
        openapi_client::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_REMOVED => {
            ConfigInstanceStatus::Removed
        }
        openapi_client::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_FAILED => {
            ConfigInstanceStatus::Failed
        }
    }
}

pub fn convert_status_storage_to_sdk(
    status: ConfigInstanceStatus,
) -> openapi_server::models::ConfigInstanceStatus {
    match status {
        ConfigInstanceStatus::Created => {
            openapi_server::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_CREATED
        }
        ConfigInstanceStatus::Queued => {
            openapi_server::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_QUEUED
        }
        ConfigInstanceStatus::Deployed => {
            openapi_server::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_DEPLOYED
        }
        ConfigInstanceStatus::Removed => {
            openapi_server::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_REMOVED
        }
        ConfigInstanceStatus::Failed => {
            openapi_server::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_FAILED
        }
    }
}

// =============================== CONFIG INSTANCE ================================= //
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ConfigInstance {
    #[serde(rename = "config_instance_id")]
    pub id: String,
    pub target_status: ConfigInstanceTargetStatus,
    pub status: ConfigInstanceStatus,
    pub filepath: Option<String>,
    pub patch_id: Option<String>,
    pub created_by_id: Option<String>,
    pub created_at: String,
    pub updated_by_id: Option<String>,
    pub updated_at: String,
    pub device_id: String,
    pub config_schema_id: String,
    pub config_instance: serde_json::Value,

    // agent specific fields
    pub config_type_slug: String,
    pub config_schema_digest: String,
}

pub fn convert_cfg_inst_backend_to_storage(
    backend_instance: openapi_client::models::BackendConfigInstance,
    config_type_slug: String,
    config_schema_digest: String,
) -> ConfigInstance {
    ConfigInstance {
        id: backend_instance.id,
        target_status: convert_target_status_backend_to_storage(backend_instance.target_status),
        status: convert_status_backend_to_storage(backend_instance.status),
        filepath: backend_instance.filepath,
        patch_id: backend_instance.patch_id,
        created_at: backend_instance.created_at,
        updated_at: backend_instance.updated_at,
        created_by_id: backend_instance.created_by_id,
        updated_by_id: backend_instance.updated_by_id,
        device_id: backend_instance.device_id,
        config_schema_id: backend_instance.config_schema_id,
        config_instance: backend_instance.instance.unwrap_or_default(),
        config_type_slug,
        config_schema_digest,
    }
}

pub fn convert_cfg_inst_storage_to_sdk(
    storage_instance: ConfigInstance,
) -> openapi_server::models::BaseConfigInstance {
    openapi_server::models::BaseConfigInstance {
        object: openapi_server::models::base_config_instance::Object::ConfigInstance,
        id: storage_instance.id,
        target_status: convert_target_status_storage_to_sdk(storage_instance.target_status),
        status: convert_status_storage_to_sdk(storage_instance.status),
        filepath: storage_instance.filepath,
        patch_id: storage_instance.patch_id,
        created_by_id: storage_instance.created_by_id,
        updated_by_id: storage_instance.updated_by_id,
        created_at: storage_instance.created_at,
        updated_at: storage_instance.updated_at,
        device_id: storage_instance.device_id,
        config_schema_id: storage_instance.config_schema_id,
        instance: Some(storage_instance.config_instance),
    }
}
