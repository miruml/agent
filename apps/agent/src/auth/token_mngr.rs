// standard library
use std::future::Future;
use std::sync::Arc;

// internal crates
use crate::auth::errors::{
    AuthCryptErr, AuthErr, AuthFileSysErr, AuthHTTPErr, ReceiveActorMessageErr,
    SendActorMessageErr, SerdeErr, TimestampConversionErr,
};
use crate::crypt::{base64, rsa};
use crate::errors::MiruError;
use crate::filesys::{cached_file::CachedFile, file::File, path::PathExt};
use crate::http::devices::DevicesExt;
use crate::storage::token::Token;
use crate::trace;
use crate::utils::time_delta_to_positive_duration;
use openapi_client::models::{IssueDeviceClaims, IssueDeviceTokenRequest};

// external crates
use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tracing::{debug, error, info};
use uuid::Uuid;

pub type TokenFile = CachedFile<Token>;

#[derive(Serialize)]
struct IssueTokenClaim {
    pub device_id: String,
    pub nonce: String,
    pub expiration: i64,
}

// ======================== SINGLE THREADED IMPLEMENTATION ========================= //
struct SingleThreadTokenManager<HTTPClientT: DevicesExt> {
    device_id: String,
    http_client: Arc<HTTPClientT>,
    token_file: CachedFile<Token>,
    private_key_file: File,
}

impl<HTTPClientT: DevicesExt> SingleThreadTokenManager<HTTPClientT> {
    fn new(
        device_id: String,
        http_client: Arc<HTTPClientT>,
        token_file: CachedFile<Token>,
        private_key_file: File,
    ) -> Result<Self, AuthErr> {
        token_file.file.assert_exists().map_err(|e| {
            AuthErr::FileSysErr(AuthFileSysErr {
                source: e,
                trace: trace!(),
            })
        })?;
        private_key_file.assert_exists().map_err(|e| {
            AuthErr::FileSysErr(AuthFileSysErr {
                source: e,
                trace: trace!(),
            })
        })?;
        Ok(Self {
            device_id,
            http_client,
            token_file,
            private_key_file,
        })
    }

    async fn get_token(&self) -> Arc<Token> {
        // get the token
        self.token_file.read()
    }

    async fn refresh_token(&mut self) -> Result<(), AuthErr> {
        // attempt to issue a new token
        let token = self.issue_token().await?;

        // update the token file
        self.token_file.write(token).await.map_err(|e| {
            AuthErr::FileSysErr(AuthFileSysErr {
                source: e,
                trace: trace!(),
            })
        })?;

        Ok(())
    }

    async fn issue_token(&self) -> Result<Token, AuthErr> {
        // prepare the token request
        let payload = self.prepare_issue_token_request().await?;

        // send the token request
        let resp = self
            .http_client
            .issue_device_token(&self.device_id, &payload)
            .await
            .map_err(|e| {
                AuthErr::HTTPErr(AuthHTTPErr {
                    source: e,
                    trace: trace!(),
                })
            })?;

        // format the response
        let expires_at = resp.expires_at.parse::<DateTime<Utc>>().map_err(|e| {
            AuthErr::TimestampConversionErr(TimestampConversionErr {
                msg: format!(
                    "failed to parse date time '{}' from string: {}",
                    resp.expires_at, e
                ),
                trace: trace!(),
            })
        })?;
        Ok(Token {
            token: resp.token,
            expires_at,
        })
    }

    async fn prepare_issue_token_request(&self) -> Result<IssueDeviceTokenRequest, AuthErr> {
        // prepare the claims
        let nonce = Uuid::new_v4().to_string();
        let expiration = Utc::now() + Duration::minutes(2);
        let claims = IssueTokenClaim {
            device_id: self.device_id.to_string(),
            nonce: nonce.clone(),
            expiration: expiration.timestamp(),
        };

        // serialize the claims into a JSON byte vector
        let claims_bytes = serde_json::to_vec(&claims).map_err(|e| {
            AuthErr::SerdeErr(SerdeErr {
                source: e,
                trace: trace!(),
            })
        })?;

        // sign the claims
        let signature_bytes = rsa::sign(&self.private_key_file, &claims_bytes)
            .await
            .map_err(|e| {
                AuthErr::CryptErr(AuthCryptErr {
                    source: e,
                    trace: trace!(),
                })
            })?;
        let signature = base64::encode_bytes_standard(&signature_bytes);

        // convert it to the http client format
        let claims = IssueDeviceClaims {
            device_id: self.device_id.to_string(),
            nonce,
            expiration: expiration.to_rfc3339(),
        };

        Ok(IssueDeviceTokenRequest {
            claims: Box::new(claims),
            signature,
        })
    }
}

// ========================= MULTI-THREADED IMPLEMENTATION ========================= //
enum WorkerCommand {
    GetToken {
        respond_to: oneshot::Sender<Result<Arc<Token>, AuthErr>>,
    },
    RefreshToken {
        respond_to: oneshot::Sender<Result<(), AuthErr>>,
    },
    Shutdown {
        respond_to: oneshot::Sender<Result<(), AuthErr>>,
    },
}

struct Worker<HTTPClientT: DevicesExt> {
    token_mngr: SingleThreadTokenManager<HTTPClientT>,
    receiver: Receiver<WorkerCommand>,
}

impl<HTTPClientT: DevicesExt> Worker<HTTPClientT> {
    async fn run(mut self) {
        while let Some(cmd) = self.receiver.recv().await {
            match cmd {
                WorkerCommand::Shutdown { respond_to } => {
                    if let Err(e) = respond_to.send(Ok(())) {
                        error!("Actor failed to send shutdown response: {:?}", e);
                    }
                    break;
                }
                WorkerCommand::GetToken { respond_to } => {
                    debug!("Worker getting token");
                    let token = self.token_mngr.get_token().await;
                    debug!("Worker got Token: {:?}", token);
                    respond_to.send(Ok(token)).unwrap();
                }
                WorkerCommand::RefreshToken { respond_to } => {
                    debug!("Worker refreshing token");
                    let result = self.token_mngr.refresh_token().await;
                    debug!("Worker refreshed Token: {:?}", result);
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to refresh token: {:?}", e);
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct TokenManager {
    sender: Sender<WorkerCommand>,
}

impl TokenManager {
    pub fn spawn<HTTPClientT: DevicesExt + 'static>(
        device_id: String,
        http_client: Arc<HTTPClientT>,
        token_file: CachedFile<Token>,
        private_key_file: File,
    ) -> Result<(Self, JoinHandle<()>), AuthErr> {
        let (sender, receiver) = mpsc::channel(32);
        let worker = Worker {
            token_mngr: SingleThreadTokenManager::new(
                device_id,
                http_client,
                token_file,
                private_key_file,
            )?,
            receiver,
        };
        let worker_handle = tokio::spawn(worker.run());
        Ok((Self { sender }, worker_handle))
    }

    pub async fn shutdown(&self) -> Result<(), AuthErr> {
        info!("Shutting down token manager...");
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::Shutdown { respond_to: send })
            .await
            .map_err(|e| {
                AuthErr::SendActorMessageErr(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                })
            })?;
        recv.await.map_err(|e| {
            AuthErr::ReceiveActorMessageErr(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            })
        })??;
        info!("Token manager shutdown complete");
        Ok(())
    }

    pub async fn get_token(&self) -> Result<Arc<Token>, AuthErr> {
        debug!("Requesting token from token manager");
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::GetToken { respond_to: send })
            .await
            .map_err(|e| {
                AuthErr::SendActorMessageErr(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                })
            })?;
        recv.await.map_err(|e| {
            AuthErr::ReceiveActorMessageErr(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            })
        })?
    }

    pub async fn refresh_token(&self) -> Result<(), AuthErr> {
        debug!("Requesting token refresh from token manager");
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::RefreshToken { respond_to: send })
            .await
            .map_err(|e| {
                AuthErr::SendActorMessageErr(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                })
            })?;
        recv.await.map_err(|e| {
            AuthErr::ReceiveActorMessageErr(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            })
        })?
    }
}

// =============================== REFRESH LOOP ==================================== //
pub async fn run_refresh_loop(
    token_mngr: Arc<TokenManager>,
    expiration_threshold: tokio::time::Duration,
    cooldown: tokio::time::Duration,
    shutdown_signal: impl Future<Output = ()> + Send + 'static,
) {
    let mut shutdown = Box::pin(shutdown_signal);

    // we want the first refresh to occur immediately if the token is expired
    let mut sleep_duration = determine_refresh_loop_sleep_duration(
        &token_mngr,
        expiration_threshold,
        tokio::time::Duration::from_secs(0),
    )
    .await;

    loop {
        tokio::select! {
            // exit if the shutdown signal is received
            _ = shutdown.as_mut() => {
                info!("Token refresh loop shutdown complete");
                return;
            }
            // else sleep and wait to refresh the token
            _ = tokio::time::sleep(sleep_duration) => {
            },
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
            determine_refresh_loop_sleep_duration(&token_mngr, expiration_threshold, cooldown)
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

pub async fn determine_refresh_loop_sleep_duration(
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
            let sleep_duration = time_delta_to_positive_duration(duration_until_expiration);
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
