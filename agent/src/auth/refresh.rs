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
use tokio::task::JoinHandle;
use tracing::{error, info};

pub async fn run_token_refresh_loop(
    token_mngr: Arc<TokenManager>,
    cooldown: Duration,
    shutdown_signal: impl Future<Output = ()> + Send + 'static,
) -> JoinHandle<()> {

    tokio::spawn(async move {
        let mut shutdown = Box::pin(shutdown_signal);

        loop {
            let mut sleep_duration: Duration;
            match token_mngr.refresh_token().await {
                Ok(_) => {
                    info!("Successfully refreshed token");

                    let token = token_mngr.get_token().await.unwrap();

                    // determine the expiration time of the token
                    let expiration = token.expires_at;
                    let duration_until_expiration = expiration - Utc::now();

                    // wait until 10 minutes before expiration to refresh the token
                    sleep_duration = time_delta_to_duration(duration_until_expiration);
                    sleep_duration -= Duration::from_secs(10*60);
                    sleep_duration = max(cooldown, sleep_duration);
                }
                Err(e) => {
                    error!("Error refreshing token: {:?}", e);
                    sleep_duration = cooldown;
                }
            }

            tokio::select! {
                _ = tokio::time::sleep(sleep_duration) => {
                    continue;
                }
                _ = shutdown.as_mut() => {
                    break;
                }
            }
        }
    })
}
