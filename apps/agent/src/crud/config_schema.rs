// internal crates
use crate::models::config_schema::ConfigSchema;

// queries
pub fn matches_config_type_slug_and_schema_digest(
    instance: &ConfigSchema,
    config_type_slug: &str,
    config_schema_digest: &str,
) -> bool {
    let instance_slug= match &instance.config_type_slug {
        Some(config_type_slug) => config_type_slug,
        None => return false,
    };
    instance.digest == config_schema_digest && instance_slug == config_type_slug
}

