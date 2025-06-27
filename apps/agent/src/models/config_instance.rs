// internal crates
use crate::deserialize_error;

// external crates
use chrono::{DateTime, TimeDelta, Utc};
use serde::Deserialize;
use serde::Serialize;
use tracing::error;
use uuid::Uuid;

// =============================== TARGET STATUS ================================== //
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConfigInstanceTargetStatus {
    #[default]
    Created,
    Deployed,
    Removed,
}

pub fn convert_target_status_backend_to_storage(
    target_status: &openapi_client::models::ConfigInstanceTargetStatus,
) -> ConfigInstanceTargetStatus {
    match target_status {
        openapi_client::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_CREATED => ConfigInstanceTargetStatus::Created,
        openapi_client::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_DEPLOYED => ConfigInstanceTargetStatus::Deployed,
        openapi_client::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_REMOVED => ConfigInstanceTargetStatus::Removed,
    }
}

pub fn convert_target_status_storage_to_sdk(
    target_status: &ConfigInstanceTargetStatus,
) -> openapi_server::models::ConfigInstanceTargetStatus {
    match target_status {
        ConfigInstanceTargetStatus::Created => openapi_server::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_CREATED,
        ConfigInstanceTargetStatus::Deployed => openapi_server::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_DEPLOYED,
        ConfigInstanceTargetStatus::Removed => openapi_server::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_REMOVED,
    }
}

// ================================== STATUS ======================================= //
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConfigInstanceStatus {
    #[default]
    Created,
    Queued,
    Deployed,
    Removed,
    Failed,
    Retrying,
}

pub fn convert_status_backend_to_storage(
    status: &openapi_client::models::ConfigInstanceStatus,
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
        openapi_client::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_RETRYING => {
            ConfigInstanceStatus::Retrying
        }
    }
}

pub fn convert_status_storage_to_sdk(
    status: &ConfigInstanceStatus,
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
        ConfigInstanceStatus::Retrying => {
            openapi_server::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_RETRYING
        }
    }
}

// ============================== ACTIVITY STATUS ================================== //
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConfigInstanceActivityStatus {
    #[default]
    Created,
    Queued,
    Deployed,
    Removed,
}

pub fn convert_activity_status_backend_to_storage(
    activity_status: &openapi_client::models::ConfigInstanceActivityStatus,
) -> ConfigInstanceActivityStatus {
    match activity_status {
        openapi_client::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_CREATED => ConfigInstanceActivityStatus::Created,
        openapi_client::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_QUEUED => ConfigInstanceActivityStatus::Queued,
        openapi_client::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_DEPLOYED => ConfigInstanceActivityStatus::Deployed,
        openapi_client::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_REMOVED => ConfigInstanceActivityStatus::Removed,
    }
}

pub fn convert_activity_status_storage_to_sdk(
    activity_status: &ConfigInstanceActivityStatus,
) -> openapi_server::models::ConfigInstanceActivityStatus {
    match activity_status {
        ConfigInstanceActivityStatus::Created => openapi_server::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_CREATED,
        ConfigInstanceActivityStatus::Queued => openapi_server::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_QUEUED,
        ConfigInstanceActivityStatus::Deployed => openapi_server::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_DEPLOYED,
        ConfigInstanceActivityStatus::Removed => openapi_server::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_REMOVED,
    }
}

// =============================== ERROR STATUS ==================================== //
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum ConfigInstanceErrorStatus {
    #[default]
    None,
    Failed,
    Retrying,
}

pub fn convert_error_status_backend_to_storage(
    error_status: &openapi_client::models::ConfigInstanceErrorStatus,
) -> ConfigInstanceErrorStatus {
    match error_status {
        openapi_client::models::ConfigInstanceErrorStatus::CONFIG_INSTANCE_ERROR_STATUS_NONE => ConfigInstanceErrorStatus::None,
        openapi_client::models::ConfigInstanceErrorStatus::CONFIG_INSTANCE_ERROR_STATUS_FAILED => ConfigInstanceErrorStatus::Failed,
        openapi_client::models::ConfigInstanceErrorStatus::CONFIG_INSTANCE_ERROR_STATUS_RETRYING => ConfigInstanceErrorStatus::Retrying,
    }
}

pub fn convert_error_status_storage_to_sdk(
    error_status: &ConfigInstanceErrorStatus,
) -> openapi_server::models::ConfigInstanceErrorStatus {
    match error_status {
        ConfigInstanceErrorStatus::None => {
            openapi_server::models::ConfigInstanceErrorStatus::CONFIG_INSTANCE_ERROR_STATUS_NONE
        }
        ConfigInstanceErrorStatus::Failed => {
            openapi_server::models::ConfigInstanceErrorStatus::CONFIG_INSTANCE_ERROR_STATUS_FAILED
        }
        ConfigInstanceErrorStatus::Retrying => {
            openapi_server::models::ConfigInstanceErrorStatus::CONFIG_INSTANCE_ERROR_STATUS_RETRYING
        }
    }
}

// =============================== CONFIG INSTANCE ================================= //
pub type ConfigInstanceID = String;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ConfigInstance {
    pub id: String,
    pub target_status: ConfigInstanceTargetStatus,
    pub activity_status: ConfigInstanceActivityStatus,
    pub error_status: ConfigInstanceErrorStatus,
    pub filepath: Option<String>,
    pub patch_id: Option<String>,
    pub created_by_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_by_id: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub device_id: String,
    pub config_schema_id: String,

    // cache fields
    pub config_type_slug: Option<String>,
    pub config_schema_digest: Option<String>,

    // fsm fields
    pub attempts: u32,
    pub cooldown_ends_at: DateTime<Utc>,
}

impl Default for ConfigInstance {
    fn default() -> Self {
        Self {
            id: format!("unknown-{}", Uuid::new_v4()),
            target_status: ConfigInstanceTargetStatus::Created,
            activity_status: ConfigInstanceActivityStatus::Created,
            error_status: ConfigInstanceErrorStatus::None,
            filepath: None,
            patch_id: None,
            created_by_id: None,
            created_at: DateTime::<Utc>::UNIX_EPOCH,
            updated_by_id: None,
            updated_at: DateTime::<Utc>::UNIX_EPOCH,
            device_id: format!("unknown-{}", Uuid::new_v4()),
            config_schema_id: format!("unknown-{}", Uuid::new_v4()),
            config_type_slug: None,
            config_schema_digest: None,
            attempts: 0,
            cooldown_ends_at: DateTime::<Utc>::UNIX_EPOCH,
        }
    }
}

impl ConfigInstance {
    pub fn status(&self) -> ConfigInstanceStatus {
        match self.error_status {
            ConfigInstanceErrorStatus::None => match self.activity_status {
                ConfigInstanceActivityStatus::Created => ConfigInstanceStatus::Created,
                ConfigInstanceActivityStatus::Queued => ConfigInstanceStatus::Queued,
                ConfigInstanceActivityStatus::Deployed => ConfigInstanceStatus::Deployed,
                ConfigInstanceActivityStatus::Removed => ConfigInstanceStatus::Removed,
            },
            ConfigInstanceErrorStatus::Failed => ConfigInstanceStatus::Failed,
            ConfigInstanceErrorStatus::Retrying => ConfigInstanceStatus::Retrying,
        }
    }

    pub fn set_cooldown(&mut self, duration: TimeDelta) {
        self.cooldown_ends_at = Utc::now() + duration;
    }

    pub fn clear_cooldown(&mut self) {
        self.cooldown_ends_at = DateTime::<Utc>::UNIX_EPOCH;
    }

    pub fn is_cooling_down(&self) -> bool {
        self.cooldown_ends_at > Utc::now()
    }
}

impl<'de> Deserialize<'de> for ConfigInstance {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        pub struct DeserializeConfigInstance {
            // required fields
            id: String,
            target_status: ConfigInstanceTargetStatus,
            activity_status: ConfigInstanceActivityStatus,
            error_status: ConfigInstanceErrorStatus,
            device_id: String,
            config_schema_id: String,

            // reasonable default fields
            created_at: Option<DateTime<Utc>>,
            updated_at: Option<DateTime<Utc>>,
            attempts: Option<u32>,
            cooldown_ends_at: Option<DateTime<Utc>>,

            // optional fields
            filepath: Option<String>,
            patch_id: Option<String>,
            created_by_id: Option<String>,
            updated_by_id: Option<String>,
            config_type_slug: Option<String>,
            config_schema_digest: Option<String>,
        }

        let result = match DeserializeConfigInstance::deserialize(deserializer) {
            Ok(instance) => instance,
            Err(e) => {
                error!("Error deserializing config instance: {}", e);
                return Err(e);
            }
        };

        let default = ConfigInstance::default();

        let created_at = result.created_at.unwrap_or(deserialize_error!(
            "config_instance",
            "created_at",
            default.created_at
        ));
        let updated_at = result.updated_at.unwrap_or(deserialize_error!(
            "config_instance",
            "updated_at",
            default.updated_at
        ));
        let attempts = result.attempts.unwrap_or(deserialize_error!(
            "config_instance",
            "attempts",
            default.attempts
        ));
        let cooldown_ends_at = result.cooldown_ends_at.unwrap_or(deserialize_error!(
            "config_instance",
            "cooldown_ends_at",
            default.cooldown_ends_at
        ));

        Ok(ConfigInstance {
            id: result.id,
            target_status: result.target_status,
            activity_status: result.activity_status,
            error_status: result.error_status,
            filepath: result.filepath,
            patch_id: result.patch_id,
            created_by_id: result.created_by_id,
            created_at,
            updated_by_id: result.updated_by_id,
            updated_at,
            device_id: result.device_id,
            config_schema_id: result.config_schema_id,
            config_type_slug: result.config_type_slug,
            config_schema_digest: result.config_schema_digest,
            attempts,
            cooldown_ends_at,
        })
    }
}

pub fn convert_cfg_inst_backend_to_storage(
    backend_instance: openapi_client::models::BackendConfigInstance,
    config_type_slug: Option<String>,
    config_schema_digest: Option<String>,
) -> ConfigInstance {
    ConfigInstance {
        id: backend_instance.id,
        target_status: convert_target_status_backend_to_storage(&backend_instance.target_status),
        activity_status: convert_activity_status_backend_to_storage(
            &backend_instance.activity_status,
        ),
        error_status: convert_error_status_backend_to_storage(&backend_instance.error_status),
        filepath: backend_instance.filepath,
        patch_id: backend_instance.patch_id,
        created_at: backend_instance.created_at.parse::<DateTime<Utc>>().unwrap_or_else(
            |e| {
                error!("Error parsing created_at: {}", e);
                DateTime::<Utc>::UNIX_EPOCH
            },
        ),
        updated_at: backend_instance.updated_at.parse::<DateTime<Utc>>().unwrap_or_else(
            |e| {
                error!("Error parsing updated_at: {}", e);
                DateTime::<Utc>::UNIX_EPOCH
            },
        ),
        created_by_id: backend_instance.created_by_id,
        updated_by_id: backend_instance.updated_by_id,
        device_id: backend_instance.device_id,
        config_schema_id: backend_instance.config_schema_id,

        // cache fields
        config_type_slug,
        config_schema_digest,

        // fsm fields
        attempts: 0,
        cooldown_ends_at: DateTime::<Utc>::UNIX_EPOCH,
    }
}

pub fn convert_cfg_inst_storage_to_sdk(
    instance: ConfigInstance,
    instance_data: serde_json::Value,
) -> openapi_server::models::BaseConfigInstance {
    let status = convert_status_storage_to_sdk(&instance.status());
    openapi_server::models::BaseConfigInstance {
        object: openapi_server::models::base_config_instance::Object::ConfigInstance,
        id: instance.id,
        target_status: convert_target_status_storage_to_sdk(&instance.target_status),
        status,
        activity_status: convert_activity_status_storage_to_sdk(&instance.activity_status),
        error_status: convert_error_status_storage_to_sdk(&instance.error_status),
        filepath: instance.filepath,
        patch_id: instance.patch_id,
        created_by_id: instance.created_by_id,
        updated_by_id: instance.updated_by_id,
        created_at: instance.created_at.to_rfc3339(),
        updated_at: instance.updated_at.to_rfc3339(),
        device_id: instance.device_id,
        config_schema_id: instance.config_schema_id,
        instance: Some(instance_data),
    }
}
