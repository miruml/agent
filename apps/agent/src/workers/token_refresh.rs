// standard library
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

// internal crates
use crate::auth::{
    errors::*,
    token_mngr::TokenManager,
};
use crate::errors::*;
use crate::utils::as_duration;

// external crates
use chrono::Utc;
use tracing::{error, info};


#[derive(Debug, Clone, Copy)]
pub struct TokenRefreshWorkerOptions {
    pub expiration_threshold: tokio::time::Duration,
    pub cooldown: tokio::time::Duration,
}

impl Default for TokenRefreshWorkerOptions {
    fn default() -> Self {
        Self {
            expiration_threshold: tokio::time::Duration::from_secs(300),
            cooldown: tokio::time::Duration::from_secs(10),
        }
    }
}


pub async fn run_token_refresh_worker(
    options: &TokenRefreshWorkerOptions,
    token_mngr: Arc<TokenManager>,
    mut shutdown_signal: Pin<Box<impl Future<Output = ()> + Send + 'static>>,
) {

    // we want the first refresh to occur immediately if the token is expired
    let mut sleep_duration = calc_refresh_delay(
        &token_mngr,
        options.expiration_threshold,
        tokio::time::Duration::from_secs(0),
    )
    .await;

    loop {
        tokio::select! {
            // exit if the shutdown signal is received
            _ = shutdown_signal.as_mut() => {
                info!("Token refresh worker shutdown complete");
                return;
            }
            // sleep and wait to refresh the token
            _ = tokio::time::sleep(sleep_duration) => {},
        }

        // refresh the token
        match refresh_token(&token_mngr, 3, tokio::time::Duration::from_millis(100)).await {
            Ok(_) => {
                info!("Token refreshed successfully");
            }
            Err(e) => {
                if e.is_network_connection_error() {
                    error!("Unable to refresh token due to a network connection error");
                } else {
                    error!("Error refreshing token: {:#?}", e);
                }
            }
        }

        // determine how long to sleep before refreshing the token.
        sleep_duration =
            calc_refresh_delay(
                &token_mngr,
                options.expiration_threshold,
                options.cooldown,
            )
            .await;
    }
}

async fn refresh_token(
    token_mngr: &TokenManager,
    attempts: usize,
    retry_delay: tokio::time::Duration,
) -> Result<(), AuthErr> {
    for i in 0..attempts {
        match token_mngr.refresh_token().await {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                if i == attempts - 1 {
                    return Err(e);
                }
            }
        }
        tokio::time::sleep(retry_delay).await;
    }
    Ok(())
}

pub async fn calc_refresh_delay(
    token_mngr: &TokenManager,
    expiration_threshold: tokio::time::Duration,
    cooldown: tokio::time::Duration,
) -> tokio::time::Duration {
    match token_mngr.get_token().await {
        Ok(token) => {
            // determine the expiration time of the token
            let expiration = token.expires_at;
            let duration_until_expiration = expiration - Utc::now();

            // attempt to refresh token when expiration is within the threshold
            let sleep_duration = as_duration(duration_until_expiration);
            if sleep_duration <= expiration_threshold + cooldown {
                cooldown
            } else {
                sleep_duration - expiration_threshold
            }
        }
        Err(e) => {
            error!("Error fetching token from token manager: {:#?}", e);
            cooldown
        }
    }
}