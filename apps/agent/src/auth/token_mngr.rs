// standard library
use std::sync::Arc;

// internal crates
use crate::auth::{
    errors::*,
    token::Token,
};
use crate::crypt::{base64, rsa};
use crate::filesys::{cached_file::CachedFile, file::File, path::PathExt};
use crate::http::{client::HTTPClient, devices::DevicesExt};
use crate::trace;
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

// =================================== TRAIT ======================================= //
#[allow(async_fn_in_trait)]
pub trait TokenManagerExt {
    async fn shutdown(&self) -> Result<(), AuthErr>;
    async fn get_token(&self) -> Result<Arc<Token>, AuthErr>;
    async fn refresh_token(&self) -> Result<(), AuthErr>;
}

// ======================== SINGLE THREADED IMPLEMENTATION ========================= //
pub struct SingleThreadTokenManager<HTTPClientT: DevicesExt> {
    device_id: String,
    http_client: Arc<HTTPClientT>,
    token_file: CachedFile<Token>,
    private_key_file: File,
}

impl<HTTPClientT: DevicesExt> SingleThreadTokenManager<HTTPClientT> {
    pub fn new(
        device_id: String,
        http_client: Arc<HTTPClientT>,
        token_file: CachedFile<Token>,
        private_key_file: File,
    ) -> Result<Self, AuthErr> {
        token_file.file.assert_exists().map_err(|e| {
            AuthErr::FileSysErr(Box::new(AuthFileSysErr {
                source: e,
                trace: trace!(),
            }))
        })?;
        private_key_file.assert_exists().map_err(|e| {
            AuthErr::FileSysErr(Box::new(AuthFileSysErr {
                source: e,
                trace: trace!(),
            }))
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
            AuthErr::FileSysErr(Box::new(AuthFileSysErr {
                source: e,
                trace: trace!(),
            }))
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
                AuthErr::HTTPErr(Box::new(AuthHTTPErr {
                    source: e,
                    trace: trace!(),
                }))
            })?;

        // format the response
        let expires_at = resp.expires_at.parse::<DateTime<Utc>>().map_err(|e| {
            AuthErr::TimestampConversionErr(Box::new(TimestampConversionErr {
                msg: format!(
                    "failed to parse date time '{}' from string: {}",
                    resp.expires_at, e
                ),
                trace: trace!(),
            }))
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
            AuthErr::SerdeErr(Box::new(SerdeErr {
                source: e,
                trace: trace!(),
            }))
        })?;

        // sign the claims
        let signature_bytes = rsa::sign(&self.private_key_file, &claims_bytes)
            .await
            .map_err(|e| {
                AuthErr::CryptErr(Box::new(AuthCryptErr {
                    source: e,
                    trace: trace!(),
                }))
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
pub enum WorkerCommand {
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

pub struct Worker<HTTPClientT: DevicesExt> {
    token_mngr: SingleThreadTokenManager<HTTPClientT>,
    receiver: Receiver<WorkerCommand>,
}

impl<HTTPClientT: DevicesExt> Worker<HTTPClientT> {
    pub fn new(
        token_mngr: SingleThreadTokenManager<HTTPClientT>,
        receiver: Receiver<WorkerCommand>,
    ) -> Self {
        Self {
            token_mngr,
            receiver,
        }
    }
}

impl<HTTPClientT: DevicesExt> Worker<HTTPClientT> {
    pub async fn run(mut self) {
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
    pub fn spawn(
        buffer_size: usize,
        device_id: String,
        http_client: Arc<HTTPClient>,
        token_file: CachedFile<Token>,
        private_key_file: File,
    ) -> Result<(Self, JoinHandle<()>), AuthErr> {
        let (sender, receiver) = mpsc::channel(buffer_size);
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

    pub fn new(sender: Sender<WorkerCommand>) -> Self {
        Self { sender }
    }
}

impl TokenManagerExt for TokenManager {
    async fn shutdown(&self) -> Result<(), AuthErr> {
        info!("Shutting down token manager...");
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::Shutdown { respond_to: send })
            .await
            .map_err(|e| {
                AuthErr::SendActorMessageErr(Box::new(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                }))
            })?;
        recv.await.map_err(|e| {
            AuthErr::ReceiveActorMessageErr(Box::new(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            }))
        })??;
        info!("Token manager shutdown complete");
        Ok(())
    }

    async fn get_token(&self) -> Result<Arc<Token>, AuthErr> {
        debug!("Requesting token from token manager");
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::GetToken { respond_to: send })
            .await
            .map_err(|e| {
                AuthErr::SendActorMessageErr(Box::new(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                }))
            })?;
        recv.await.map_err(|e| {
            AuthErr::ReceiveActorMessageErr(Box::new(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            }))
        })?
    }

    async fn refresh_token(&self) -> Result<(), AuthErr> {
        debug!("Requesting token refresh from token manager");
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::RefreshToken { respond_to: send })
            .await
            .map_err(|e| {
                AuthErr::SendActorMessageErr(Box::new(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                }))
            })?;
        recv.await.map_err(|e| {
            AuthErr::ReceiveActorMessageErr(Box::new(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            }))
        })?
    }
}

impl TokenManagerExt for Arc<TokenManager> {
    async fn shutdown(&self) -> Result<(), AuthErr> {
        self.as_ref().shutdown().await
    }

    async fn get_token(&self) -> Result<Arc<Token>, AuthErr> {
        self.as_ref().get_token().await
    }

    async fn refresh_token(&self) -> Result<(), AuthErr> {
        self.as_ref().refresh_token().await
    }
}