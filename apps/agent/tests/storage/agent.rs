// internal crates
use config_agent::filesys::dir::Dir;
use config_agent::models::agent::Agent;
use config_agent::storage::agent::assert_activated;
use config_agent::storage::errors::StorageErr;

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
