// internal crates
use crate::cache::{entry::CacheEntry, file::FileCache};
use crate::models::config_schema::{ConfigSchema, ConfigSchemaID};

// config schema cache
pub type ConfigSchemaCacheEntry = CacheEntry<ConfigSchemaID, ConfigSchema>;
pub type ConfigSchemaCache = FileCache<ConfigSchemaID, ConfigSchema>;
