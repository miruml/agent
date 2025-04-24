// standard library
use std::cmp::max;

// internal crates
use crate::auth::token_mngr::TokenManager;
use crate::utils::time_delta_to_duration;

// external crates
use chrono::Utc;
use tokio::time::Duration;
use tracing::{error, info};

pub async fn run_token_refresh_loop(
    token_mngr: &TokenManager,
    cooldown: Duration
) {
    loop {
        match token_mngr.refresh_token().await {
            Ok(_) => {
                info!("Successfully refreshed token");

                let token = token_mngr.get_token().await.unwrap();

                // determine the expiration time of the token
                let expiration = token.expires_at;
                let duration_until_expiration = expiration - Utc::now();

                // wait until 10 minutes before expiration to refresh the token
                let mut sleep_duration = time_delta_to_duration(duration_until_expiration);
                sleep_duration -= Duration::from_secs(10*60);
                sleep_duration = max(cooldown, sleep_duration);

                // sleep for the duration
                tokio::time::sleep(sleep_duration).await;
            }
            Err(e) => {
                error!("Error refreshing token: {:?}", e);
                
                // cooldown before retrying
                tokio::time::sleep(cooldown).await;
            }
        }
    }
}
