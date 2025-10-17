// internal crates
use crate::models::config_schema::ConfigSchema;

// queries
pub fn matches_config_type_slug_and_schema_digest(
    cfg_sch: &ConfigSchema,
    config_type_slug: &str,
    config_schema_digest: &str,
) -> bool {
    let cfg_sch_slug = match &cfg_sch.config_type_slug {
        Some(config_type_slug) => config_type_slug,
        None => return false,
    };
    cfg_sch.digest == config_schema_digest && cfg_sch_slug == config_type_slug
}
