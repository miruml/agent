// standard library
use std::sync::Arc;

// internal crates
use crate::http::mock::MockAuthClient;
use config_agent::auth::errors::AuthErr;
use config_agent::auth::{
    token_mngr::{
        determine_refresh_loop_sleep_duration, run_refresh_loop, SingleThreadTokenManager,
        TokenFile, TokenManager, Worker,
    },
    token::Token,
};
use config_agent::crypt::rsa;
use config_agent::filesys::{cached_file::CachedFile, dir::Dir, file::File};
use config_agent::http::{
    client::HTTPClient,
    errors::{HTTPErr, MockErr},
};
use config_agent::storage::token::Token;
use config_agent::trace;
use openapi_client::models::TokenResponse;

// external crates
use chrono::{Duration, Utc};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub fn spawn(
    buffer_size: usize,
    device_id: String,
    http_client: Arc<MockAuthClient>,
    token_file: CachedFile<Token>,
    private_key_file: File,
) -> Result<(TokenManager, JoinHandle<()>), AuthErr> {
    let (sender, receiver) = mpsc::channel(buffer_size);
    let worker = Worker::new(
        SingleThreadTokenManager::new(device_id, http_client, token_file, private_key_file)?,
        receiver,
    );
    let worker_handle = tokio::spawn(worker.run());
    Ok((TokenManager::new(sender), worker_handle))
}

pub mod spawn {
    use super::*;

    #[tokio::test]
    async fn token_file_does_not_exist() {
        // create and delete the token file
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let token_file = TokenFile::new_with_default(dir.file("token.json"), Token::default())
            .await
            .unwrap();
        token_file.file.delete().await.unwrap();

        // spawn the token manager
        let http_client = HTTPClient::new("doesntmatter").await;
        let result = TokenManager::spawn(
            32,
            "device_id".to_string(),
            Arc::new(http_client),
            token_file,
            File::new("private_key.pem"),
        )
        .unwrap_err();
        assert!(matches!(result, AuthErr::FileSysErr(_)));
    }

    #[tokio::test]
    async fn private_key_file_does_not_exist() {
        // create the token file
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let token_file = TokenFile::new_with_default(dir.file("token.json"), Token::default())
            .await
            .unwrap();
        token_file.file.delete().await.unwrap();

        // create and delete the private key file
        let private_key_file = dir.file("private_key.pem");
        private_key_file.delete().await.unwrap();

        // spawn the token manager
        let http_client = HTTPClient::new("doesntmatter").await;
        let result = TokenManager::spawn(
            32,
            "device_id".to_string(),
            Arc::new(http_client),
            token_file,
            File::new("private_key.pem"),
        )
        .unwrap_err();

        assert!(matches!(result, AuthErr::FileSysErr(_)));
    }

    #[tokio::test]
    async fn success() {
        // create the token file
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let token_file = TokenFile::new_with_default(dir.file("token.json"), Token::default())
            .await
            .unwrap();
        let private_key_file = dir.file("private_key.pem");
        let public_key_file = dir.file("public_key.pem");
        rsa::gen_key_pair(4096, &private_key_file, &public_key_file, true)
            .await
            .unwrap();

        let http_client = HTTPClient::new("doesntmatter").await;
        TokenManager::spawn(
            32,
            "device_id".to_string(),
            Arc::new(http_client),
            token_file,
            private_key_file,
        )
        .unwrap();
    }
}

pub mod shutdown {
    use super::*;

    #[tokio::test]
    async fn shutdown() {
        // create the token file
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let token_file = TokenFile::new_with_default(dir.file("token.json"), Token::default())
            .await
            .unwrap();
        let private_key_file = dir.file("private_key.pem");
        let public_key_file = dir.file("public_key.pem");
        rsa::gen_key_pair(4096, &private_key_file, &public_key_file, true)
            .await
            .unwrap();

        let mock_http_client = MockAuthClient::default();
        let (token_mngr, worker_handle) = spawn(
            32,
            "device_id".to_string(),
            Arc::new(mock_http_client),
            token_file,
            private_key_file,
        )
        .unwrap();
        token_mngr.shutdown().await.unwrap();
        worker_handle.await.unwrap();
    }
}

pub mod get_token {
    use super::*;

    #[tokio::test]
    async fn success() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let token_file = TokenFile::new_with_default(dir.file("token.json"), Token::default())
            .await
            .unwrap();
        let private_key_file = dir.file("private_key.pem");
        private_key_file
            .write_string("private_key", false, false)
            .await
            .unwrap();

        let mock_http_client = MockAuthClient::default();

        let (token_mngr, _) = spawn(
            32,
            "device_id".to_string(),
            Arc::new(mock_http_client),
            token_file,
            private_key_file,
        )
        .unwrap();

        let token = token_mngr.get_token().await.unwrap();
        assert_eq!(token.as_ref(), &Token::default());
    }
}

pub mod refresh_token {
    use super::*;

    #[tokio::test]
    async fn invalid_private_key() {
        // prepare the arguments
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let token_file = TokenFile::new_with_default(dir.file("token.json"), Token::default())
            .await
            .unwrap();
        let private_key_file = dir.file("private_key.pem");
        private_key_file
            .write_string("private_key", false, false)
            .await
            .unwrap();

        // prepare the mock http client
        let mut mock_http_client = MockAuthClient::default();
        let expected = TokenResponse {
            token: "token".to_string(),
            expires_at: Utc::now().to_rfc3339(),
        };
        mock_http_client.issue_device_token_result = Box::new(move || Ok(expected.clone()));

        // spawn the token manager
        let (token_mngr, _) = spawn(
            32,
            "device_id".to_string(),
            Arc::new(mock_http_client),
            token_file,
            private_key_file,
        )
        .unwrap();

        // refresh the token
        let result = token_mngr.refresh_token().await.unwrap_err();
        assert!(matches!(result, AuthErr::CryptErr(_)));
    }

    #[tokio::test]
    async fn http_client_error() {
        // prepare the arguments
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let token_file = TokenFile::new_with_default(dir.file("token.json"), Token::default())
            .await
            .unwrap();
        let private_key_file = dir.file("private_key.pem");
        let public_key_file = dir.file("public_key.pem");
        rsa::gen_key_pair(4096, &private_key_file, &public_key_file, true)
            .await
            .unwrap();

        // prepare the mock http client
        let mock_http_client = MockAuthClient {
            issue_device_token_result: Box::new(move || {
                Err(HTTPErr::MockErr(Box::new(MockErr {
                    is_network_connection_error: false,
                    trace: trace!(),
                })))
            }),
            ..Default::default()
        };

        // spawn the token manager
        let (token_mngr, _) = spawn(
            32,
            "device_id".to_string(),
            Arc::new(mock_http_client),
            token_file,
            private_key_file,
        )
        .unwrap();

        // refresh the token
        let result = token_mngr.refresh_token().await.unwrap_err();
        assert!(matches!(result, AuthErr::HTTPErr(_)));
    }

    #[tokio::test]
    async fn success() {
        // prepare the arguments
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let token_file = TokenFile::new_with_default(dir.file("token.json"), Token::default())
            .await
            .unwrap();
        let private_key_file = dir.file("private_key.pem");
        let public_key_file = dir.file("public_key.pem");
        rsa::gen_key_pair(4096, &private_key_file, &public_key_file, true)
            .await
            .unwrap();

        // prepare the mock http client
        let expires_at = Utc::now() + Duration::days(1);
        let resp = TokenResponse {
            token: "token".to_string(),
            expires_at: expires_at.to_rfc3339(),
        };
        let resp_clone = resp.clone();
        let mock_http_client = MockAuthClient {
            issue_device_token_result: Box::new(move || Ok(resp_clone.clone())),
            ..Default::default()
        };

        // spawn the token manager
        let (token_mngr, _) = spawn(
            32,
            "device_id".to_string(),
            Arc::new(mock_http_client),
            token_file,
            private_key_file,
        )
        .unwrap();

        // refresh the token
        token_mngr.refresh_token().await.unwrap();
        let token = token_mngr.get_token().await.unwrap();
        let expected = Token {
            token: resp.token,
            expires_at,
        };
        assert_eq!(token.as_ref(), &expected);
    }

    #[tokio::test]
    async fn success_token_file_deleted() {
        // prepare the arguments
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let token_file = dir.file("token.json");
        let cached_token_file = TokenFile::new_with_default(token_file.clone(), Token::default())
            .await
            .unwrap();
        let private_key_file = dir.file("private_key.pem");
        let public_key_file = dir.file("public_key.pem");
        rsa::gen_key_pair(4096, &private_key_file, &public_key_file, true)
            .await
            .unwrap();

        // prepare the mock http client
        let expires_at = Utc::now() + Duration::days(1);
        let resp = TokenResponse {
            token: "token".to_string(),
            expires_at: expires_at.to_rfc3339(),
        };
        let resp_clone = resp.clone();
        let mock_http_client = MockAuthClient {
            issue_device_token_result: Box::new(move || Ok(resp_clone.clone())),
            ..Default::default()
        };

        // spawn the token manager
        let (token_mngr, _) = spawn(
            32,
            "device_id".to_string(),
            Arc::new(mock_http_client),
            cached_token_file,
            private_key_file,
        )
        .unwrap();

        // delete the token file just because it should still work
        token_file.delete().await.unwrap();

        // refresh the token
        token_mngr.refresh_token().await.unwrap();
        let token = token_mngr.get_token().await.unwrap();
        let expected = Token {
            token: resp.token,
            expires_at,
        };
        assert_eq!(token.as_ref(), &expected);
    }
}

async fn create_token_manager(dir: &Dir, token: Option<Token>) -> (TokenManager, JoinHandle<()>) {
    // prepare the token manager args
    let token_file = dir.file("token.json");
    let cached_token_file =
        TokenFile::new_with_default(token_file.clone(), token.unwrap_or_default())
            .await
            .unwrap();
    let private_key_file = dir.file("private_key.pem");
    let public_key_file = dir.file("public_key.pem");
    rsa::gen_key_pair(4096, &private_key_file, &public_key_file, true)
        .await
        .unwrap();

    // prepare the mock http client
    let expires_at = Utc::now() + Duration::days(1);
    let resp = TokenResponse {
        token: "token".to_string(),
        expires_at: expires_at.to_rfc3339(),
    };
    let resp_clone = resp.clone();
    let mock_http_client = MockAuthClient {
        issue_device_token_result: Box::new(move || Ok(resp_clone.clone())),
        ..Default::default()
    };

    // spawn the token manager
    let (token_mngr, worker_handle) = spawn(
        32,
        "device_id".to_string(),
        Arc::new(mock_http_client),
        cached_token_file,
        private_key_file,
    )
    .unwrap();

    (token_mngr, worker_handle)
}

pub mod run_refresh_loop {
    use super::*;

    #[tokio::test]
    async fn run_and_shutdown() {
        // testing this is actually pretty difficult because it's an infinite loop
        // so we'll just test that it can be spawned and shutdown

        // create the token manager
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let (token_mngr, worker_handle) = create_token_manager(&dir, None).await;
        let token_mngr = Arc::new(token_mngr);

        // run the refresh loop with shutdown signal
        let (shutdown_tx, _shutdown_rx): (tokio::sync::broadcast::Sender<()>, _) =
            tokio::sync::broadcast::channel(1);
        let mut shutdown_rx = shutdown_tx.subscribe();
        let shutdown_signal = async move {
            let _ = shutdown_rx.recv().await;
        };

        // run the refresh loop
        let token_mngr_for_spawn = token_mngr.clone();
        let token_refresh_handle = tokio::spawn(async move {
            run_refresh_loop(
                token_mngr_for_spawn,
                tokio::time::Duration::from_secs(60),
                tokio::time::Duration::from_secs(30),
                shutdown_signal,
            )
            .await;
        });

        // shutdown the token manager and refresh loop
        shutdown_tx.send(()).unwrap();
        token_mngr.shutdown().await.unwrap();
        worker_handle.await.unwrap();
        token_refresh_handle.await.unwrap();
    }
}

pub mod determine_refresh_loop_sleep_duration {
    use super::*;

    #[tokio::test]
    async fn expired_in_past() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let token = Token {
            token: "token".to_string(),
            expires_at: Utc::now() - Duration::minutes(60),
        };
        let (token_mngr, _) = create_token_manager(&dir, Some(token)).await;

        let ten_minutes = tokio::time::Duration::from_secs(10 * 60);
        let cooldown = tokio::time::Duration::from_secs(30);
        let sleep_duration =
            determine_refresh_loop_sleep_duration(&token_mngr, ten_minutes, cooldown).await;
        assert_eq!(sleep_duration, cooldown);
    }

    #[tokio::test]
    async fn expires_less_than_10_minutes() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let token = Token {
            token: "token".to_string(),
            expires_at: Utc::now() + Duration::minutes(9),
        };
        let (token_mngr, _) = create_token_manager(&dir, Some(token)).await;

        let ten_minutes = tokio::time::Duration::from_secs(10 * 60);
        let cooldown = tokio::time::Duration::from_secs(30);
        let sleep_duration =
            determine_refresh_loop_sleep_duration(&token_mngr, ten_minutes, cooldown).await;
        assert_eq!(sleep_duration, cooldown);
    }

    #[tokio::test]
    async fn expires_more_than_10_minutes() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let token = Token {
            token: "token".to_string(),
            expires_at: Utc::now() + Duration::minutes(35),
        };
        let (token_mngr, _) = create_token_manager(&dir, Some(token)).await;

        let ten_minutes = tokio::time::Duration::from_secs(10 * 60);
        let cooldown = tokio::time::Duration::from_secs(30);
        let sleep_duration =
            determine_refresh_loop_sleep_duration(&token_mngr, ten_minutes, cooldown).await;

        // expect to wait until 10 minutes before expiration (25 minutes)
        let expected = tokio::time::Duration::from_secs(25 * 60);
        assert!(sleep_duration < expected);
        assert!(sleep_duration > expected - tokio::time::Duration::from_secs(5));
    }
}
