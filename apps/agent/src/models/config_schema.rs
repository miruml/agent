// internal crates
use crate::deserialize_error;

// external crates
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;

// config schema
pub type ConfigSchemaID = String;

// config schema
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ConfigSchema {
    pub id: String,
    pub version: i32,
    pub digest: String,
    pub created_at: String,
    pub created_by_id: Option<String>,
    pub config_type_id: String,

    // cache fields
    pub config_type_slug: Option<String>,
}

impl Default for ConfigSchema {
    fn default() -> Self {
        Self {
            id: format!("unknown-{}", Uuid::new_v4()),
            version: -1,
            digest: format!("unknown-{}", Uuid::new_v4()),
            created_at: DateTime::<Utc>::UNIX_EPOCH.to_rfc3339(),
            created_by_id: None,
            config_type_id: format!("unknown-{}", Uuid::new_v4()),
            config_type_slug: None,
        }
    }
}

impl<'de> Deserialize<'de> for ConfigSchema {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        pub struct DeserializeConfigSchema {
            // required fields
            id: String,
            version: i32,
            digest: String,
            config_type_id: String,

            // reasonable default fields
            created_at: Option<String>,

            // optional fields
            created_by_id: Option<String>,
            config_type_slug: Option<String>,
        }

        let result = match DeserializeConfigSchema::deserialize(deserializer) {
            Ok(schema) => schema,
            Err(e) => {
                error!("Error deserializing config schema: {}", e);
                return Err(e);
            }
        };

        let default = ConfigSchema::default();

        let created_at = result.created_at.unwrap_or(deserialize_error!(
            "config_schema",
            "created_at",
            default.created_at
        ));

        Ok(ConfigSchema {
            id: result.id,
            version: result.version,
            digest: result.digest,
            created_at,
            created_by_id: result.created_by_id,
            config_type_id: result.config_type_id,
            config_type_slug: result.config_type_slug,
        })
    }
}
