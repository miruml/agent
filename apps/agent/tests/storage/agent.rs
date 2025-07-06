// internal crates
use config_agent::filesys::dir::Dir;
use config_agent::storage::{
    agent::{Agent, assert_activated},
    errors::StorageErr,
};

// external crates
use serde_json::json;

pub mod assert_activated {
    use super::*;

    #[tokio::test]
    async fn file_does_not_exist() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let agent_file = dir.file("agent.json");

        let result = assert_activated(&agent_file).await.unwrap_err();
        assert!(matches!(result, StorageErr::FileSysErr { .. }));
    }

    #[tokio::test]
    async fn invalid_file_contents() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let agent_file = dir.file("agent.json");
        agent_file
            .write_string("not a valid agent", true, true)
            .await
            .unwrap();

        let result = assert_activated(&agent_file).await.unwrap_err();
        assert!(matches!(result, StorageErr::FileSysErr { .. }));
    }

    #[tokio::test]
    async fn agent_not_activated() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let agent_file = dir.file("agent.json");
        let agent = Agent {
            activated: false,
            ..Default::default()
        };
        agent_file.write_json(&agent, true, true).await.unwrap();

        let result = assert_activated(&agent_file).await.unwrap_err();
        assert!(matches!(result, StorageErr::AgentNotActivatedErr { .. }));
    }
}

#[test]
fn serialize_deserialize_agent() {
    let agent = Agent {
        device_id: "dvc_123".to_string(),
        activated: true,
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
    };
    let valid_input = json!({
        "device_id": expected.device_id,
        "activated": expected.activated,
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