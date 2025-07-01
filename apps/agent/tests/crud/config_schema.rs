// internal crates
use config_agent::crud::config_schema::matches_config_type_slug_and_schema_digest;
use config_agent::models::config_schema::ConfigSchema;

pub mod matches_config_type_slug_and_schema_digest {
    use super::*;

    #[test]
    fn matches() {
        let config_type_slug = "slug";
        let digest = "digest";
        let config_schema = ConfigSchema {
            config_type_slug: Some(config_type_slug.to_string()),
            digest: digest.to_string(),
            ..Default::default()
        };
        assert!(matches_config_type_slug_and_schema_digest(
            &config_schema,
            config_type_slug,
            digest
        ));
    }

    #[test]
    fn doesnt_match_config_type_slug() {
        let config_type_slug = "slug";
        let digest = "digest";
        let config_schema = ConfigSchema {
            config_type_slug: Some("wrong_slug".to_string()),
            digest: digest.to_string(),
            ..Default::default()
        };
        assert!(!matches_config_type_slug_and_schema_digest(
            &config_schema,
            config_type_slug,
            digest
        ));
    }

    #[test]
    fn doesnt_match_schema_digest() {
        let config_type_slug = "slug";
        let digest = "digest";
        let config_schema = ConfigSchema {
            config_type_slug: Some(config_type_slug.to_string()),
            digest: "wrong_digest".to_string(),
            ..Default::default()
        };
        assert!(!matches_config_type_slug_and_schema_digest(
            &config_schema,
            config_type_slug,
            digest
        ));
    }
}
