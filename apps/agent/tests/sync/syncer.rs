// standard crates
use std::sync::Arc;

// internal crates
use config_agent::auth::token_mngr::{
    TokenFile,
    TokenManager,
};
use config_agent::deploy::fsm;
use config_agent::filesys::dir::Dir;
use config_agent::http::client::HTTPClient;
use config_agent::storage::token::Token;
use config_agent::sync::{
    errors::SyncErr,
    syncer::{SingleThreadSyncer, Syncer, Worker},
};
use crate::auth::token_mngr::spawn as spawn_token_manager;
use crate::http::mock::{MockAuthClient, MockConfigInstancesClient};

// external crates
use chrono::{DateTime, Utc};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub async fn create_token_manager(
    dir: &Dir,
    http_client: Arc<MockAuthClient>,
) -> (TokenManager, JoinHandle<()>) {
    let token_file = TokenFile::new_with_default(dir.file("token.json"), Token::default())
        .await
        .unwrap();
    let private_key_file = dir.file("private_key.pem");
    private_key_file.write_string("private_key", true, true).await.unwrap();

    spawn_token_manager(
        32,
        "device_id".to_string(),
        http_client.clone(),
        token_file,
        private_key_file,
    ).unwrap()
}

pub fn spawn(
    buffer_size: usize,
    device_id: String,
    http_client: Arc<MockConfigInstancesClient>,
    token_mngr: Arc<TokenManager>,
    deployment_dir: Dir,
    fsm_settings: fsm::Settings,
) -> Result<(Syncer, JoinHandle<()>), SyncErr> {
    let (sender, receiver) = mpsc::channel(buffer_size);
    let worker = Worker::new(
        SingleThreadSyncer::new(
            device_id,
            http_client,
            token_mngr,
            deployment_dir,
            fsm_settings,
        ),
        receiver,
    );
    let worker_handle = tokio::spawn(worker.run());
    Ok((Syncer::new(sender), worker_handle))
}

pub mod shutdown {
    use super::*;

    #[tokio::test]
    async fn shutdown() {
        let dir = Dir::create_temp_dir("spawn").await.unwrap();
        let auth_client = Arc::new(MockAuthClient::default());
        let (token_mngr, _) = create_token_manager(&dir, auth_client.clone()).await;

        let http_client = Arc::new(HTTPClient::new("doesntmatter").await);
        let (syncer, worker_handler) = Syncer::spawn(
            32,
            "device_id".to_string(),
            http_client.clone(),
            Arc::new(token_mngr),
            dir,
            fsm::Settings::default(),
        ).unwrap();

        syncer.shutdown().await.unwrap();
        worker_handler.await.unwrap();
    }
}

pub mod sync {
    use super::*;

    #[tokio::test]
    async fn network_error_ignored() {
    }

    #[tokio::test]
    async fn network_error_not_ignored() {
    }

    #[tokio::test]
    async fn already_synced() {
    }

    #[tokio::test]
    async fn pull_and_deploy_and_push_new_instance() {
    }

}

pub mod get_last_synced_at {
    use super::*;

    #[tokio::test]
    async fn get_last_synced_at() {
        let dir = Dir::create_temp_dir("spawn").await.unwrap();
        let auth_client = Arc::new(MockAuthClient::default());
        let (token_mngr, _) = create_token_manager(&dir, auth_client.clone()).await;

        let http_client = Arc::new(MockConfigInstancesClient::default());
        let (syncer, _) = spawn(
            32,
            "device_id".to_string(),
            http_client.clone(),
            Arc::new(token_mngr),
            dir,
            fsm::Settings::default(),
        ).unwrap();

        assert_eq!(syncer.get_last_synced_at().await.unwrap(), DateTime::<Utc>::UNIX_EPOCH);
    }
}
