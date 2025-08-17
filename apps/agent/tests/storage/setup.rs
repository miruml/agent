// internal crates
use config_agent::filesys::{dir::Dir, file::File, path::PathExt};
use config_agent::storage::settings::Settings;
use config_agent::storage::{agent::Agent, layout::StorageLayout, setup::clean_storage_setup};

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

        // config instance deployment directory
        let config_instance_deployment_dir = layout.config_instance_deployment_dir();
        assert!(config_instance_deployment_dir.exists());
    }

    async fn create_temp_key_files(layout: &StorageLayout) -> (File, File) {
        let temp_dir = layout.temp_dir();
        let private_key_file = temp_dir.file("private_key.pem");
        private_key_file
            .write_string("test", true, true)
            .await
            .unwrap();
        let public_key_file = temp_dir.file("public_key.pem");
        public_key_file
            .write_string("test", true, true)
            .await
            .unwrap();

        (private_key_file, public_key_file)
    }

    #[tokio::test]
    async fn src_public_key_file_doesnt_exist() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);
        let settings = Settings::default();

        // create the public / private key files
        let (private_key_file, public_key_file) = create_temp_key_files(&layout).await;
        public_key_file.delete().await.unwrap();

        // setup the storage
        let agent = Agent::default();
        clean_storage_setup(
            &layout,
            &agent,
            &settings,
            &private_key_file,
            &public_key_file,
        )
        .await
        .unwrap_err();
    }

    #[tokio::test]
    async fn src_private_key_file_doesnt_exist() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);
        let settings = Settings::default();

        // create the public / private key files
        let (private_key_file, public_key_file) = create_temp_key_files(&layout).await;
        private_key_file.delete().await.unwrap();

        // setup the storage
        let agent = Agent::default();
        clean_storage_setup(
            &layout,
            &agent,
            &settings,
            &private_key_file,
            &public_key_file,
        )
        .await
        .unwrap_err();
    }

    #[tokio::test]
    async fn clean_install() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);
        let settings = Settings::default();

        // create the public / private key files
        let (private_key_file, public_key_file) = create_temp_key_files(&layout).await;

        // setup the storage
        let agent = Agent::default();
        clean_storage_setup(
            &layout,
            &agent,
            &settings,
            &private_key_file,
            &public_key_file,
        )
        .await
        .unwrap();

        // validate the storage
        validate_storage(&layout).await;
    }

    #[tokio::test]
    async fn agent_file_already_exists() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);
        let settings = Settings::default();

        // create the public / private key files
        let (private_key_file, public_key_file) = create_temp_key_files(&layout).await;

        // create the agent file
        let agent_file = layout.agent_file();
        agent_file
            .write_json(&Agent::default(), true, true)
            .await
            .unwrap();

        // setup the storage
        let agent = Agent::default();
        clean_storage_setup(
            &layout,
            &agent,
            &settings,
            &private_key_file,
            &public_key_file,
        )
        .await
        .unwrap();

        // validate the storage
        validate_storage(&layout).await;
    }

    #[tokio::test]
    async fn auth_directory_already_exists() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);

        // create the public / private key files
        let (private_key_file, public_key_file) = create_temp_key_files(&layout).await;

        // create the auth directory
        let auth_dir = layout.auth_dir();
        auth_dir.root.create(false).await.unwrap();

        // setup the storage
        let agent = Agent::default();
        let settings = Settings::default();
        clean_storage_setup(
            &layout,
            &agent,
            &settings,
            &private_key_file,
            &public_key_file,
        )
        .await
        .unwrap();

        // validate the storage
        validate_storage(&layout).await;
    }

    #[tokio::test]
    async fn private_key_file_already_exists() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);

        // create the public / private key files
        let (private_key_file, public_key_file) = create_temp_key_files(&layout).await;

        // setup the storage
        let agent = Agent::default();
        let settings = Settings::default();
        clean_storage_setup(
            &layout,
            &agent,
            &settings,
            &private_key_file,
            &public_key_file,
        )
        .await
        .unwrap();

        // validate the storage
        validate_storage(&layout).await;
    }

    #[tokio::test]
    async fn public_key_file_already_exists() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);

        // create the public / private key files
        let (private_key_file, public_key_file) = create_temp_key_files(&layout).await;

        // setup the storage
        let agent = Agent::default();
        let settings = Settings::default();
        clean_storage_setup(
            &layout,
            &agent,
            &settings,
            &private_key_file,
            &public_key_file,
        )
        .await
        .unwrap();

        // validate the storage
        validate_storage(&layout).await;
    }

    #[tokio::test]
    async fn caches_directory_already_exists() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);
        let settings = Settings::default();

        // create the public / private key files
        let (private_key_file, public_key_file) = create_temp_key_files(&layout).await;

        // create the caches directory
        let caches_dir = layout.caches_dir();
        let subfile = caches_dir.file("test");
        subfile.write_string("test", true, true).await.unwrap();
        assert!(subfile.exists());

        // setup the storage
        let agent = Agent::default();
        clean_storage_setup(
            &layout,
            &agent,
            &settings,
            &private_key_file,
            &public_key_file,
        )
        .await
        .unwrap();

        // validate the storage
        validate_storage(&layout).await;

        // subfile should be deleted
        assert!(!subfile.exists());
    }

    #[tokio::test]
    async fn config_instance_deployment_directory_already_exists() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);
        let settings = Settings::default();

        // create the public / private key files
        let (private_key_file, public_key_file) = create_temp_key_files(&layout).await;

        // create the config instance deployment directory
        let config_instance_deployment_dir = layout.config_instance_deployment_dir();
        let subfile = config_instance_deployment_dir.file("test");
        subfile.write_string("test", true, true).await.unwrap();
        assert!(subfile.exists());

        // setup the storage
        let agent = Agent::default();
        clean_storage_setup(
            &layout,
            &agent,
            &settings,
            &private_key_file,
            &public_key_file,
        )
        .await
        .unwrap();

        // validate the storage
        validate_storage(&layout).await;

        // subfile should be deleted
        assert!(!subfile.exists());
    }
}
