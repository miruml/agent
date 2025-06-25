// standard crates
use std::fmt::Display;

// internal crates
use crate::models::config_schema::ConfigSchema;
use crate::storage::cache::Cache;

// external crates
use serde::Deserialize;
use serde::Serialize;

// config instance cache
pub type ConfigSchemaCache = Cache<ConfigSchemaCacheKey, ConfigSchema>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ConfigSchemaCacheKey {
    pub config_schema_id: String,
}

impl Display for ConfigSchemaCacheKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.config_schema_id)
    }
}

