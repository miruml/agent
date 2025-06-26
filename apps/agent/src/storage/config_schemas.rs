// internal crates
use crate::models::config_schema::ConfigSchema;
use crate::storage::cache::{Cache, CacheEntry};

// config schema cache
pub type ConfigSchemaID = String;
pub type ConfigSchemaCache = Cache<ConfigSchemaID, ConfigSchema>;

// queries
pub fn config_type_slug_and_schema_digest_filter(
    entry: &CacheEntry<ConfigSchemaID, ConfigSchema>,
    config_type_slug: &str,
    config_schema_digest: &str,
) -> bool {
    let entry_slug= match &entry.value.config_type_slug {
        Some(config_type_slug) => config_type_slug,
        None => return false,
    };
    entry.value.digest == config_schema_digest && entry_slug == config_type_slug
}

