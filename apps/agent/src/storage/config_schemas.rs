// internal crates
use crate::models::config_schema::{
    ConfigSchema,
    ConfigSchemaID,
};
use crate::cache::{
    entry::CacheEntry,
    file::FileCache,
};

// config schema cache
pub type ConfigSchemaCacheEntry = CacheEntry<ConfigSchemaID, ConfigSchema>;
pub type ConfigSchemaCache = FileCache<ConfigSchemaID, ConfigSchema>;

