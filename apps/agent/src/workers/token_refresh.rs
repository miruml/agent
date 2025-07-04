// standard library
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

// internal crates
use crate::auth::token_mngr::TokenManagerExt;
use crate::errors::*;
use crate::utils::calc_exp_backoff;
use crate::workers::cooldown::CooldownOptions;

// external crates
use chrono::Utc;
use tracing::{debug, error, info};


#[derive(Debug, Clone)]
pub struct TokenRefreshWorkerOptions {
    pub refresh_advance_secs: i64,
    pub polling: CooldownOptions,
}

impl Default for TokenRefreshWorkerOptions {
    fn default() -> Self {
        Self {
            refresh_advance_secs: 60 * 15, // 15 minutes
            polling: CooldownOptions {
                base_secs: 10,
                growth_factor: 2,
                max_secs: 60 * 60, // 1 hour
            },
        }
    }
}

pub async fn run_token_refresh_worker<F, Fut, TokenManagerT: TokenManagerExt>(
    options: &TokenRefreshWorkerOptions,
    token_mngr: &TokenManagerT,
    sleep_func: F, // for testing purposes
    mut shutdown_signal: Pin<Box<impl Future<Output = ()> + Send + 'static>>,
) where 
    F: Fn(Duration) -> Fut,
    Fut: Future<Output = ()> + Send
{
    let mut wait: Duration;
    let mut err_streak = 0;

    loop {
        // refresh
        match token_mngr.refresh_token().await {
            Ok(_) => {
                if err_streak > 0 {
                    info!("Token refreshed successfully after an error streak of {} errors", err_streak);
                } else {
                    info!("Token refreshed successfully");
                }
                err_streak = 0;
            }
            Err(e) => {
                if e.is_network_connection_error() {
                    debug!("Unable to refresh token due to a network connection error: {:#?}", e);
                } else {
                    error!("Error refreshing token (error streak: {}): {:#?}", err_streak, e);
                    err_streak += 1;
                }
            }
        }

        wait = calc_refresh_wait(
            token_mngr,
            options.refresh_advance_secs,
            err_streak,
            options.polling,
        )
        .await;

        let refresh_time = Utc::now() + wait;
        info!("Waiting until {:?} to refresh token", refresh_time);

        // wait to refresh or shutdown if the signal is received
        tokio::select! {
            _ = shutdown_signal.as_mut() => {
                info!("Token refresh worker shutdown complete");
                return;
            }
            _ = sleep_func(wait) => {},
        }
    }
}

pub async fn calc_refresh_wait<TokenManagerT: TokenManagerExt>(
    token_mngr: &TokenManagerT,
    refresh_advance_secs: i64,
    err_streak: u32,
    cooldown: CooldownOptions,
) -> Duration {

    // calculate the cooldown period
    let cooldown_secs = calc_exp_backoff(
        cooldown.base_secs,
        cooldown.growth_factor,
        err_streak,
        cooldown.max_secs
    );

    match token_mngr.get_token().await {
        Ok(token) => {
            let expiration = token.expires_at;
            let secs_until_exp = (expiration - Utc::now()).num_seconds();

            // if the token will expire within our refresh advance period, only wait
            // for the cooldown period before refreshing the token
            if secs_until_exp < refresh_advance_secs {
                Duration::from_secs(cooldown_secs as u64)

            // if the token expires after our refresh advance period, wait until the
            // refresh advance period begins to refresh the token
            } else {
                Duration::from_secs((secs_until_exp - refresh_advance_secs) as u64)
            }
        }
        Err(e) => {
            error!("Error fetching token from token manager: {:#?}", e);
            Duration::from_secs(cooldown_secs as u64)
        }
    }
}