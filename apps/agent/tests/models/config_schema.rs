// internal crates
use config_agent::models::config_schema::ConfigSchema;

// external crates
use serde_json::json;
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

#[test]
fn deserialize_config_schema() {
    // valid deserialization
    let expected = ConfigSchema {
        id: "cfg_123".to_string(),
        version: 1,
        digest: "digest_123".to_string(),
        created_at: "2021-01-01T00:00:00Z".to_string(),
        created_by_id: None,
        config_type_id: "cfg_type_123".to_string(),
        config_type_slug: None,
    };
    let valid_input = json!({
        "id": expected.id,
        "version": expected.version,
        "digest": expected.digest,
        "created_at": expected.created_at,
        "created_by_id": expected.created_by_id,
        "config_type_id": expected.config_type_id,
        "config_type_slug": expected.config_type_slug,
    });
    let config_schema: ConfigSchema = serde_json::from_value(valid_input).unwrap();
    assert_eq!(config_schema, expected);

    // exclude required fields
    let empty_input = json!({});
    assert!(serde_json::from_value::<ConfigSchema>(empty_input).is_err());

    // exclude default fields
    let expected = ConfigSchema {
        // required fields
        id: "cfg_456".to_string(),
        version: -1,
        digest: "digest_456".to_string(),
        config_type_id: "cfg_type_456".to_string(),
        // rest are defaults
        ..Default::default()
    };
    let valid_input = json!({
        "id": expected.id,
        "version": expected.version,
        "digest": expected.digest,
        "config_type_id": expected.config_type_id,
    });
    let config_schema: ConfigSchema = serde_json::from_value(valid_input).unwrap();
    assert_eq!(config_schema, expected);

    // invalid JSON
    assert!(serde_json::from_str::<ConfigSchema>("invalid-json").is_err());
}
