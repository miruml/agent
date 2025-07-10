// internal crates
use config_agent::filesys::{dir::Dir, path::PathExt};
use config_agent::storage::settings::Settings;
use config_agent::storage::{agent::Agent, layout::StorageLayout, setup::setup_storage};

pub mod setup_storage {
    use super::*;

    async fn validate_storage(layout: &StorageLayout) {
        // agent file
        let agent_file = layout.agent_file();
        let agent_file_content = agent_file.read_json::<Agent>().await.unwrap();
        assert_eq!(agent_file_content, Agent::default());

        // settings file
        let settings_file = layout.settings_file();
        let settings_file_content = settings_file.read_json::<Settings>().await.unwrap();
        assert_eq!(settings_file_content, Settings::default());

        // token file
        let auth_layout = layout.auth_dir();
        let token_file = auth_layout.token_file();
        assert!(token_file.exists());

        // private key file
        let private_key_file = auth_layout.private_key_file();
        assert!(private_key_file.exists());
        let private_key_contents = private_key_file.read_string().await.unwrap();
        assert!(!private_key_contents.is_empty());

        // public key file
        let public_key_file = auth_layout.public_key_file();
        assert!(public_key_file.exists());
        let public_key_contents = public_key_file.read_string().await.unwrap();
        assert!(!public_key_contents.is_empty());
    }

    #[tokio::test]
    async fn clean_install() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);
        let settings = Settings::default();

        // setup the storage
        let agent = Agent::default();
        setup_storage(&layout, &agent, &settings).await.unwrap();

        // validate the storage
        validate_storage(&layout).await;
    }

    #[tokio::test]
    async fn agent_file_already_exists() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);
        let settings = Settings::default();

        // create the agent file
        let agent_file = layout.agent_file();
        agent_file
            .write_json(&Agent::default(), true, true)
            .await
            .unwrap();

        // setup the storage
        let agent = Agent::default();
        setup_storage(&layout, &agent, &settings).await.unwrap();

        // validate the storage
        validate_storage(&layout).await;
    }

    #[tokio::test]
    async fn auth_directory_already_exists() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);

        // create the auth directory
        let auth_dir = layout.auth_dir();
        auth_dir.root.create(false).await.unwrap();

        // setup the storage
        let agent = Agent::default();
        let settings = Settings::default();
        setup_storage(&layout, &agent, &settings).await.unwrap();

        // validate the storage
        validate_storage(&layout).await;
    }

    #[tokio::test]
    async fn private_key_file_already_exists() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);

        // create the private key file
        let auth_layout = layout.auth_dir();
        let private_key_file = auth_layout.private_key_file();
        private_key_file
            .write_string("test", true, true)
            .await
            .unwrap();

        // setup the storage
        let agent = Agent::default();
        let settings = Settings::default();
        setup_storage(&layout, &agent, &settings).await.unwrap();

        // validate the storage
        validate_storage(&layout).await;
    }

    #[tokio::test]
    async fn public_key_file_already_exists() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);

        // create the public key file
        let auth_layout = layout.auth_dir();
        let public_key_file = auth_layout.public_key_file();
        public_key_file
            .write_string("test", true, true)
            .await
            .unwrap();

        // setup the storage
        let agent = Agent::default();
        let settings = Settings::default();
        setup_storage(&layout, &agent, &settings).await.unwrap();

        // validate the storage
        validate_storage(&layout).await;
    }

    #[tokio::test]
    async fn caches_directory_already_exists() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);
        let settings = Settings::default();

        // create the caches directory
        let caches_dir = layout.caches_dir();
        let subfile = caches_dir.file("test");
        subfile.write_string("test", true, true).await.unwrap();
        assert!(subfile.exists());

        // setup the storage
        let agent = Agent::default();
        setup_storage(&layout, &agent, &settings).await.unwrap();

        // validate the storage
        validate_storage(&layout).await;

        // subfile should be deleted
        assert!(!subfile.exists());
    }
}
