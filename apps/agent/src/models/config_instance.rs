// internal crates
use crate::deserialize_error;

// external crates
use chrono::{DateTime, TimeDelta, Utc};
use serde::Deserialize;
use serde::Serialize;
use tracing::{error, warn};
use uuid::Uuid;

// =============================== TARGET STATUS ================================== //
#[derive(Clone, Copy, Debug, Default, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum TargetStatus {
    #[default]
    Created,
    Deployed,
    #[serde(other)]
    Removed,
}

impl<'de> Deserialize<'de> for TargetStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let default = TargetStatus::default();
        match s.as_str() {
            "created" => Ok(TargetStatus::Created),
            "deployed" => Ok(TargetStatus::Deployed),
            "removed" => Ok(TargetStatus::Removed),
            status => {
                warn!(
                    "target status '{}' is not valid, defaulting to {:?}",
                    status, default
                );
                Ok(default)
            }
        }
    }
}

impl TargetStatus {
    pub fn variants() -> Vec<TargetStatus> {
        vec![
            TargetStatus::Created,
            TargetStatus::Deployed,
            TargetStatus::Removed,
        ]
    }

    pub fn from_backend(
        target_status: &openapi_client::models::ConfigInstanceTargetStatus,
    ) -> TargetStatus {
        match target_status {
            openapi_client::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_CREATED => TargetStatus::Created,
            openapi_client::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_DEPLOYED => TargetStatus::Deployed,
            openapi_client::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_REMOVED => TargetStatus::Removed,
        }
    }

    pub fn to_backend(
        target_status: &TargetStatus,
    ) -> openapi_client::models::ConfigInstanceTargetStatus {
        match target_status {
            TargetStatus::Created => openapi_client::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_CREATED,
            TargetStatus::Deployed => openapi_client::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_DEPLOYED,
            TargetStatus::Removed => openapi_client::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_REMOVED,
        }
    }

    pub fn from_sdk(
        target_status: &openapi_server::models::ConfigInstanceTargetStatus,
    ) -> TargetStatus {
        match target_status {
            openapi_server::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_CREATED => TargetStatus::Created,
            openapi_server::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_DEPLOYED => TargetStatus::Deployed,
            openapi_server::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_REMOVED => TargetStatus::Removed,
        }
    }

    pub fn to_sdk(
        target_status: &TargetStatus,
    ) -> openapi_server::models::ConfigInstanceTargetStatus {
        match target_status {
            TargetStatus::Created => openapi_server::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_CREATED,
            TargetStatus::Deployed => openapi_server::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_DEPLOYED,
            TargetStatus::Removed => openapi_server::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_REMOVED,
        }
    }
}

// ============================== ACTIVITY STATUS ================================== //
#[derive(Clone, Copy, Debug, Default, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ActivityStatus {
    #[default]
    Created,
    Queued,
    Deployed,
    Removed,
}

impl<'de> Deserialize<'de> for ActivityStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let default = ActivityStatus::default();
        match s.as_str() {
            "created" => Ok(ActivityStatus::Created),
            "queued" => Ok(ActivityStatus::Queued),
            "deployed" => Ok(ActivityStatus::Deployed),
            "removed" => Ok(ActivityStatus::Removed),
            status => {
                warn!(
                    "activity status '{}' is not valid, defaulting to {:?}",
                    status, default
                );
                Ok(default)
            }
        }
    }
}

impl ActivityStatus {
    pub fn variants() -> Vec<ActivityStatus> {
        vec![
            ActivityStatus::Created,
            ActivityStatus::Queued,
            ActivityStatus::Deployed,
            ActivityStatus::Removed,
        ]
    }

    pub fn from_backend(
        activity_status: &openapi_client::models::ConfigInstanceActivityStatus,
    ) -> ActivityStatus {
        match activity_status {
            openapi_client::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_CREATED => ActivityStatus::Created,
            openapi_client::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_QUEUED => ActivityStatus::Queued,
            openapi_client::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_DEPLOYED => ActivityStatus::Deployed,
            openapi_client::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_REMOVED => ActivityStatus::Removed,
        }
    }

    pub fn to_backend(
        activity_status: &ActivityStatus,
    ) -> openapi_client::models::ConfigInstanceActivityStatus {
        match activity_status {
            ActivityStatus::Created => openapi_client::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_CREATED,
            ActivityStatus::Queued => openapi_client::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_QUEUED,
            ActivityStatus::Deployed => openapi_client::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_DEPLOYED,
            ActivityStatus::Removed => openapi_client::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_REMOVED,
        }
    }

    pub fn from_sdk(
        activity_status: &openapi_server::models::ConfigInstanceActivityStatus,
    ) -> ActivityStatus {
        match activity_status {
            openapi_server::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_CREATED => ActivityStatus::Created,
            openapi_server::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_QUEUED => ActivityStatus::Queued,
            openapi_server::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_DEPLOYED => ActivityStatus::Deployed,
            openapi_server::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_REMOVED => ActivityStatus::Removed,
        }
    }

    pub fn to_sdk(
        activity_status: &ActivityStatus,
    ) -> openapi_server::models::ConfigInstanceActivityStatus {
        match activity_status {
            ActivityStatus::Created => openapi_server::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_CREATED,
            ActivityStatus::Queued => openapi_server::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_QUEUED,
            ActivityStatus::Deployed => openapi_server::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_DEPLOYED,
            ActivityStatus::Removed => openapi_server::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_REMOVED,
        }
    }
}

// =============================== ERROR STATUS ==================================== //
#[derive(Clone, Copy, Debug, Default, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ErrorStatus {
    #[default]
    None,
    Failed,
    Retrying,
}

impl<'de> Deserialize<'de> for ErrorStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let default = ErrorStatus::default();
        match s.as_str() {
            "none" => Ok(ErrorStatus::None),
            "failed" => Ok(ErrorStatus::Failed),
            "retrying" => Ok(ErrorStatus::Retrying),
            status => {
                warn!(
                    "error status '{}' is not valid, defaulting to {:?}",
                    status, default
                );
                Ok(default)
            }
        }
    }
}

impl ErrorStatus {
    pub fn variants() -> Vec<ErrorStatus> {
        vec![
            ErrorStatus::None,
            ErrorStatus::Failed,
            ErrorStatus::Retrying,
        ]
    }

    pub fn from_backend(
        error_status: &openapi_client::models::ConfigInstanceErrorStatus,
    ) -> ErrorStatus {
        match error_status {
            openapi_client::models::ConfigInstanceErrorStatus::CONFIG_INSTANCE_ERROR_STATUS_NONE => ErrorStatus::None,
            openapi_client::models::ConfigInstanceErrorStatus::CONFIG_INSTANCE_ERROR_STATUS_FAILED => ErrorStatus::Failed,
            openapi_client::models::ConfigInstanceErrorStatus::CONFIG_INSTANCE_ERROR_STATUS_RETRYING => ErrorStatus::Retrying,
        }
    }

    pub fn to_backend(
        error_status: &ErrorStatus,
    ) -> openapi_client::models::ConfigInstanceErrorStatus {
        match error_status {
            ErrorStatus::None => {
                openapi_client::models::ConfigInstanceErrorStatus::CONFIG_INSTANCE_ERROR_STATUS_NONE
            }
            ErrorStatus::Failed => {
                openapi_client::models::ConfigInstanceErrorStatus::CONFIG_INSTANCE_ERROR_STATUS_FAILED
            }
            ErrorStatus::Retrying => {
                openapi_client::models::ConfigInstanceErrorStatus::CONFIG_INSTANCE_ERROR_STATUS_RETRYING
            }
        }
    }

    pub fn from_sdk(
        error_status: &openapi_server::models::ConfigInstanceErrorStatus,
    ) -> ErrorStatus {
        match error_status {
            openapi_server::models::ConfigInstanceErrorStatus::CONFIG_INSTANCE_ERROR_STATUS_NONE => ErrorStatus::None,
            openapi_server::models::ConfigInstanceErrorStatus::CONFIG_INSTANCE_ERROR_STATUS_FAILED => ErrorStatus::Failed,
            openapi_server::models::ConfigInstanceErrorStatus::CONFIG_INSTANCE_ERROR_STATUS_RETRYING => ErrorStatus::Retrying,
        }
    }

    pub fn to_sdk(error_status: &ErrorStatus) -> openapi_server::models::ConfigInstanceErrorStatus {
        match error_status {
            ErrorStatus::None => {
                openapi_server::models::ConfigInstanceErrorStatus::CONFIG_INSTANCE_ERROR_STATUS_NONE
            }
            ErrorStatus::Failed => {
                openapi_server::models::ConfigInstanceErrorStatus::CONFIG_INSTANCE_ERROR_STATUS_FAILED
            }
            ErrorStatus::Retrying => {
                openapi_server::models::ConfigInstanceErrorStatus::CONFIG_INSTANCE_ERROR_STATUS_RETRYING
            }
        }
    }
}

// ================================== STATUS ======================================= //
#[derive(Clone, Copy, Debug, Default, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    #[default]
    Created,
    Queued,
    Deployed,
    Removed,
    Failed,
    Retrying,
}

impl<'de> Deserialize<'de> for Status {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let default = Status::default();
        match s.as_str() {
            "created" => Ok(Status::Created),
            "queued" => Ok(Status::Queued),
            "deployed" => Ok(Status::Deployed),
            "removed" => Ok(Status::Removed),
            "failed" => Ok(Status::Failed),
            "retrying" => Ok(Status::Retrying),
            status => {
                warn!(
                    "status '{}' is not valid, defaulting to {:?}",
                    status, default
                );
                Ok(default)
            }
        }
    }
}

impl Status {
    pub fn variants() -> Vec<Status> {
        vec![
            Status::Created,
            Status::Queued,
            Status::Deployed,
            Status::Removed,
            Status::Failed,
            Status::Retrying,
        ]
    }

    pub fn from_backend(status: &openapi_client::models::ConfigInstanceStatus) -> Status {
        match status {
            openapi_client::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_CREATED => {
                Status::Created
            }
            openapi_client::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_QUEUED => {
                Status::Queued
            }
            openapi_client::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_DEPLOYED => {
                Status::Deployed
            }
            openapi_client::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_REMOVED => {
                Status::Removed
            }
            openapi_client::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_FAILED => {
                Status::Failed
            }
            openapi_client::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_RETRYING => {
                Status::Retrying
            }
        }
    }

    pub fn to_backend(status: &Status) -> openapi_client::models::ConfigInstanceStatus {
        match status {
            Status::Created => {
                openapi_client::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_CREATED
            }
            Status::Queued => {
                openapi_client::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_QUEUED
            }
            Status::Deployed => {
                openapi_client::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_DEPLOYED
            }
            Status::Removed => {
                openapi_client::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_REMOVED
            }
            Status::Failed => {
                openapi_client::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_FAILED
            }
            Status::Retrying => {
                openapi_client::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_RETRYING
            }
        }
    }

    pub fn from_sdk(status: &openapi_server::models::ConfigInstanceStatus) -> Status {
        match status {
            openapi_server::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_CREATED => {
                Status::Created
            }
            openapi_server::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_QUEUED => {
                Status::Queued
            }
            openapi_server::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_DEPLOYED => {
                Status::Deployed
            }
            openapi_server::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_REMOVED => {
                Status::Removed
            }
            openapi_server::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_FAILED => {
                Status::Failed
            }
            openapi_server::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_RETRYING => {
                Status::Retrying
            }
        }
    }

    pub fn to_sdk(status: &Status) -> openapi_server::models::ConfigInstanceStatus {
        match status {
            Status::Created => {
                openapi_server::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_CREATED
            }
            Status::Queued => {
                openapi_server::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_QUEUED
            }
            Status::Deployed => {
                openapi_server::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_DEPLOYED
            }
            Status::Removed => {
                openapi_server::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_REMOVED
            }
            Status::Failed => {
                openapi_server::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_FAILED
            }
            Status::Retrying => {
                openapi_server::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_RETRYING
            }
        }
    }
}

// =============================== CONFIG INSTANCE ================================= //
pub type ConfigInstanceID = String;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ConfigInstance {
    pub id: String,
    pub target_status: TargetStatus,
    pub activity_status: ActivityStatus,
    pub error_status: ErrorStatus,
    pub relative_filepath: Option<String>,
    pub patch_id: Option<String>,
    pub created_by_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_by_id: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub device_id: String,
    pub config_schema_id: String,
    pub config_type_id: String,

    // fsm fields
    pub attempts: u32,
    pub cooldown_ends_at: DateTime<Utc>,
}

impl Default for ConfigInstance {
    fn default() -> Self {
        Self {
            id: format!("unknown-{}", Uuid::new_v4()),
            target_status: TargetStatus::Created,
            activity_status: ActivityStatus::Created,
            error_status: ErrorStatus::None,
            relative_filepath: None,
            patch_id: None,
            created_by_id: None,
            created_at: DateTime::<Utc>::UNIX_EPOCH,
            updated_by_id: None,
            updated_at: DateTime::<Utc>::UNIX_EPOCH,
            device_id: format!("unknown-{}", Uuid::new_v4()),
            config_schema_id: format!("unknown-{}", Uuid::new_v4()),
            config_type_id: format!("unknown-{}", Uuid::new_v4()),
            attempts: 0,
            cooldown_ends_at: DateTime::<Utc>::UNIX_EPOCH,
        }
    }
}

impl ConfigInstance {
    pub fn from_backend(
        backend_instance: openapi_client::models::ConfigInstance,
    ) -> ConfigInstance {
        ConfigInstance {
            id: backend_instance.id,
            target_status: TargetStatus::from_backend(&backend_instance.target_status),
            activity_status: ActivityStatus::from_backend(&backend_instance.activity_status),
            error_status: ErrorStatus::from_backend(&backend_instance.error_status),
            relative_filepath: backend_instance.relative_filepath,
            patch_id: backend_instance.patch_id,
            created_at: backend_instance
                .created_at
                .parse::<DateTime<Utc>>()
                .unwrap_or_else(|e| {
                    error!("Error parsing created_at: {}", e);
                    DateTime::<Utc>::UNIX_EPOCH
                }),
            updated_at: backend_instance
                .updated_at
                .parse::<DateTime<Utc>>()
                .unwrap_or_else(|e| {
                    error!("Error parsing updated_at: {}", e);
                    DateTime::<Utc>::UNIX_EPOCH
                }),
            created_by_id: backend_instance.created_by_id,
            updated_by_id: backend_instance.updated_by_id,
            device_id: backend_instance.device_id,
            config_schema_id: backend_instance.config_schema_id,
            config_type_id: backend_instance.config_type_id,

            // fsm fields
            attempts: 0,
            cooldown_ends_at: DateTime::<Utc>::UNIX_EPOCH,
        }
    }

    pub fn to_sdk(
        cfg_inst: ConfigInstance,
        content: serde_json::Value,
    ) -> openapi_server::models::ConfigInstance {
        let status = Status::to_sdk(&cfg_inst.status());
        openapi_server::models::ConfigInstance {
            object: openapi_server::models::config_instance::Object::ConfigInstance,
            id: cfg_inst.id,
            target_status: TargetStatus::to_sdk(&cfg_inst.target_status),
            status,
            activity_status: ActivityStatus::to_sdk(&cfg_inst.activity_status),
            error_status: ErrorStatus::to_sdk(&cfg_inst.error_status),
            relative_filepath: cfg_inst.relative_filepath,
            created_at: cfg_inst.created_at.to_rfc3339(),
            updated_at: cfg_inst.updated_at.to_rfc3339(),
            config_schema_id: cfg_inst.config_schema_id,
            config_type_id: cfg_inst.config_type_id,
            config_type: None,
            content,
        }
    }

    pub fn status(&self) -> Status {
        match self.error_status {
            ErrorStatus::None => match self.activity_status {
                ActivityStatus::Created => Status::Created,
                ActivityStatus::Queued => Status::Queued,
                ActivityStatus::Deployed => Status::Deployed,
                ActivityStatus::Removed => Status::Removed,
            },
            ErrorStatus::Retrying => Status::Retrying,
            ErrorStatus::Failed => Status::Failed,
        }
    }

    pub fn set_cooldown(&mut self, duration: TimeDelta) {
        self.cooldown_ends_at = Utc::now() + duration;
    }

    pub fn clear_cooldown(&mut self) {
        self.cooldown_ends_at = DateTime::<Utc>::UNIX_EPOCH;
    }

    pub fn is_in_cooldown(&self) -> bool {
        self.cooldown_ends_at > Utc::now()
    }

    pub fn cooldown(&self) -> TimeDelta {
        self.cooldown_ends_at - Utc::now()
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
            target_status: TargetStatus,
            activity_status: ActivityStatus,
            error_status: ErrorStatus,
            device_id: String,
            config_schema_id: String,
            config_type_id: String,

            // reasonable default fields
            created_at: Option<DateTime<Utc>>,
            updated_at: Option<DateTime<Utc>>,
            attempts: Option<u32>,
            cooldown_ends_at: Option<DateTime<Utc>>,

            // optional fields
            relative_filepath: Option<String>,
            patch_id: Option<String>,
            created_by_id: Option<String>,
            updated_by_id: Option<String>,
        }

        let result = match DeserializeConfigInstance::deserialize(deserializer) {
            Ok(cfg_inst) => cfg_inst,
            Err(e) => {
                error!("Error deserializing config instance: {}", e);
                return Err(e);
            }
        };

        let default = ConfigInstance::default();

        let created_at = result.created_at.unwrap_or_else(|| {
            deserialize_error!("config_instance", "created_at", default.created_at)
        });
        let updated_at = result.updated_at.unwrap_or_else(|| {
            deserialize_error!("config_instance", "updated_at", default.updated_at)
        });
        let attempts = result
            .attempts
            .unwrap_or_else(|| deserialize_error!("config_instance", "attempts", default.attempts));
        let cooldown_ends_at = result.cooldown_ends_at.unwrap_or_else(|| {
            deserialize_error!(
                "config_instance",
                "cooldown_ends_at",
                default.cooldown_ends_at
            )
        });

        Ok(ConfigInstance {
            id: result.id,
            target_status: result.target_status,
            activity_status: result.activity_status,
            error_status: result.error_status,
            relative_filepath: result.relative_filepath,
            patch_id: result.patch_id,
            created_by_id: result.created_by_id,
            created_at,
            updated_by_id: result.updated_by_id,
            updated_at,
            device_id: result.device_id,
            config_schema_id: result.config_schema_id,
            config_type_id: result.config_type_id,
            attempts,
            cooldown_ends_at,
        })
    }
}
