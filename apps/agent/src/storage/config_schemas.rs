// internal crates
use crate::models::config_schema::{
    ConfigSchema,
    ConfigSchemaID,
};
use crate::storage::cache::Cache;

// config schema cache
pub type ConfigSchemaCache = Cache<ConfigSchemaID, ConfigSchema>;

