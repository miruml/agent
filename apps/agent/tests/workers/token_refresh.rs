// standard library
use std::sync::Arc;

// internal crates
use crate::http::mock::MockAuthClient;
use config_agent::auth::{
    token_mngr::{
        TokenFile, TokenManager,
    },
    token::Token,
};
use config_agent::crypt::rsa;
use config_agent::filesys::dir::Dir;
use openapi_client::models::TokenResponse;
use config_agent::workers::token_refresh::{
    TokenRefreshWorkerOptions,
    run_token_refresh_worker,
    calc_refresh_delay,
};

use crate::auth::token_mngr::spawn as spawn_token_mngr;

// external crates

use chrono::{Duration, Utc};
use tokio::task::JoinHandle;


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
    let (token_mngr, worker_handle) = spawn_token_mngr(
        32,
        "device_id".to_string(),
        Arc::new(mock_http_client),
        cached_token_file,
        private_key_file,
    )
    .unwrap();

    (token_mngr, worker_handle)
}

pub mod run_refresh_token_worker {
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
            run_token_refresh_worker(
                &TokenRefreshWorkerOptions::default(),
                token_mngr_for_spawn,
                Box::pin(shutdown_signal),
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

pub mod calc_refresh_delay {
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
        let sleep_duration = calc_refresh_delay(&token_mngr, ten_minutes, cooldown).await;
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
            calc_refresh_delay(&token_mngr, ten_minutes, cooldown).await;
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
        let sleep_duration = calc_refresh_delay(&token_mngr, ten_minutes, cooldown).await;

        // expect to wait until 10 minutes before expiration (25 minutes)
        let expected = tokio::time::Duration::from_secs(25 * 60);
        assert!(sleep_duration < expected);
        assert!(sleep_duration > expected - tokio::time::Duration::from_secs(5));
    }
}

