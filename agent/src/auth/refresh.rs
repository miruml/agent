// standard library
use std::cmp::max;
use std::future::Future;
use std::sync::Arc;

// internal crates
use crate::auth::token_mngr::TokenManager;
use crate::utils::time_delta_to_duration;

// external crates
use chrono::Utc;
use tokio::time::Duration;
use tracing::{error, info};

pub async fn run_token_refresh_loop(
    token_mngr: Arc<TokenManager>,
    cooldown: Duration,
    shutdown_signal: impl Future<Output = ()> + Send + 'static,
) {
    let mut shutdown = Box::pin(shutdown_signal);
    let mut sleep_duration: Duration;

    loop {
        // determine how long to sleep before refreshing the token.
        sleep_duration = determine_sleep_duration(&token_mngr, cooldown).await;

        tokio::select! {
            // exit if the shutdown signal is received
            _ = shutdown.as_mut() => {
                info!("Token refresh loop shutdown complete");
                return;
            }
            // else sleep and wait to refresh the token
            _ = tokio::time::sleep(sleep_duration) => {},
        }

        // refresh the token
        if let Err(e) = token_mngr.refresh_token().await {
            error!("Error refreshing token: {:#?}", e);
        }
    }
}

async fn determine_sleep_duration(token_mngr: &TokenManager, cooldown: Duration) -> Duration {
    match token_mngr.get_token().await {
        Ok(token) => {
            // determine the expiration time of the token
            let expiration = token.expires_at;
            let duration_until_expiration = expiration - Utc::now();

            // wait until 10 minutes before expiration to refresh the token
            let mut sleep_duration = time_delta_to_duration(duration_until_expiration);
            sleep_duration -= Duration::from_secs(10 * 60);
            sleep_duration = max(cooldown, sleep_duration);
            sleep_duration
        }
        Err(e) => {
            error!("Error fetching token from token manager: {:#?}", e);
            cooldown
        }
    }
}
