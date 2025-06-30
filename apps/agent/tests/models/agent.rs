// internal crates
use config_agent::models::agent::Agent;
use config_agent::logs::LogLevel;

// external crates
use serde_json::json;
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

#[test]
fn serialize_deserialize_agent() {
    let agent = Agent {
        device_id: "dvc_123".to_string(),
        activated: true,
        backend_base_url: "https://configs.api.miruml.com/agent/v1/arglebargle".to_string(),
        log_level: LogLevel::Debug,
        config_instance_deployment_base_path: "/srv/miru/configs/arglebargle".to_string(),
    };
    let serialized = serde_json::to_string(&agent).unwrap();
    let deserialized = serde_json::from_str::<Agent>(&serialized).unwrap();
    assert_eq!(deserialized, agent);
}

#[test]
fn deserialize_agent() {
    // valid deserialization
    let expected = Agent {
        device_id: "dvc_123".to_string(),
        activated: true,
        backend_base_url: "https://configs.api.miruml.com/agent/v1/arglebargle".to_string(),
        log_level: LogLevel::Debug,
        config_instance_deployment_base_path: "/srv/miru/configs/arglebargle".to_string(),
    };
    let valid_input = json!({
        "device_id": expected.device_id,
        "activated": expected.activated,
        "backend_base_url": expected.backend_base_url,
        "log_level": expected.log_level,
        "config_instance_deployment_base_path": expected.config_instance_deployment_base_path,
    });
    let agent: Agent = serde_json::from_value(valid_input).unwrap();
    assert_eq!(agent, expected);

    // exclude required fields
    let empty_input = json!({});
    assert!(serde_json::from_value::<Agent>(empty_input).is_err());

    // exclude default fields
    let expected = Agent {
        // required fields
        device_id: "dvc_456".to_string(),
        // rest are defaults
        ..Default::default()
    };
    let valid_input = json!({
        "device_id": expected.device_id,
    });
    let agent: Agent = serde_json::from_value(valid_input).unwrap();
    assert_eq!(agent, expected);

    // invalid JSON
    assert!(serde_json::from_str::<Agent>("invalid-json").is_err());
}
