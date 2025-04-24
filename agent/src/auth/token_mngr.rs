// standard library
use std::sync::Arc;

// internal crates
use crate::auth::errors::{
    AuthErr,
    AuthCryptErr,
    AuthFileSysErr,
    AuthHTTPErr,
    SerdeErr,
    SendActorMessageErr,
    ReceiveActorMessageErr,
    TimestampConversionErr,
};
use crate::crypt::{
    base64,
    rsa,
};
use crate::filesys::file::File;
use crate::filesys::cached_file::CachedFile;
use crate::http::auth::ClientAuthExt;
use crate::storage::token::Token;
use crate::trace;
use openapi_client::models::{
    IssueClientClaims, IssueClientTokenRequest,
};

// external crates
use chrono::{Utc, Duration, DateTime};
use serde::Serialize;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::oneshot;
use tracing::error;
use uuid::Uuid;

#[derive(Serialize)]
struct IssueTokenClaim {
    pub client_id: String,
    pub nonce: String,
    pub expiration: i64,
}

// ======================== SINGLE THREADED IMPLEMENTATION ========================= //
struct SingleThreadTokenManager<HTTPClientT: ClientAuthExt> {
    client_id: String,
    http_client: Arc<HTTPClientT>,
    token_file: CachedFile<Token>,
    private_key_file: File,
    last_refresh: Option<DateTime<Utc>>,
    refresh_cooldown: Duration,
}

impl<HTTPClientT: ClientAuthExt> SingleThreadTokenManager<HTTPClientT> {

    fn new(
        client_id: String,
        http_client: Arc<HTTPClientT>,
        token_file: CachedFile<Token>,
        private_key_file: File,
        refresh_cooldown: Duration,
    ) -> Self {
        Self {
            client_id,
            http_client,
            token_file,
            private_key_file,
            last_refresh: None,
            refresh_cooldown,
        }
    }

    async fn get_token(&self) -> Arc<Token> {
        // get the token
        self.token_file.read()
    }

    async fn refresh_token(&mut self) -> Result<(), AuthErr> {
        // exit if we are still in the cooldown period
        if let Some(last_refresh) = self.last_refresh {
            if Utc::now() < last_refresh + self.refresh_cooldown {
                return Ok(());
            }
        }

        // attempt to issue a new token
        let token = self.issue_token().await?;

        // update the token file
        self.token_file.write(token).await.map_err(|e| AuthErr::FileSysErr(AuthFileSysErr {
            source: e,
            trace: trace!(),
        }))?;

        Ok(())
    }

    async fn issue_token(&self) -> Result<Token, AuthErr> {
        // prepare the token request
        let payload = self.prepare_issue_token_request().await?;

        // send the token request
        let resp = self.http_client.issue_client_token(
            &self.client_id,
            &payload,
        ).await.map_err(|e| AuthErr::HTTPErr(AuthHTTPErr {
            source: e,
            trace: trace!(),
        }))?;

        // format the response
        let expires_at = resp.expires_at.parse::<DateTime<Utc>>().map_err(|e| AuthErr::TimestampConversionErr(TimestampConversionErr {
                msg: format!("failed to parse date time '{}' from string: {}", resp.expires_at, e),
                trace: trace!(),
            }))?;
        Ok(Token {
            token: resp.token,
            expires_at,
        })
    }

    async fn prepare_issue_token_request(&self) -> Result<IssueClientTokenRequest, AuthErr> {
        // prepare the claims
        let nonce = Uuid::new_v4().to_string();
        let expiration = Utc::now() + Duration::minutes(2);
        let claims = IssueTokenClaim {
            client_id: self.client_id.to_string(),
            nonce: nonce.clone(),
            expiration: expiration.timestamp(),
        };

        // serialize the claims into a JSON byte vector
        let claims_bytes = serde_json::to_vec(&claims).map_err(|e| AuthErr::SerdeErr(SerdeErr {
            source: e,
            trace: trace!(),
        }))?;

        // sign the claims
        let signature_bytes = rsa::sign(&self.private_key_file, &claims_bytes).await.map_err(|e| AuthErr::CryptErr(AuthCryptErr{
            source: e,
            trace: trace!(),
        }))?;
        let signature = base64::encode_bytes_standard(&signature_bytes);

        // convert it to the http client format
        let claims = IssueClientClaims {
            client_id: self.client_id.to_string(),
            nonce,
            expiration: expiration.to_rfc3339(),
        };

        Ok(IssueClientTokenRequest {
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
}

struct Worker<HTTPClientT: ClientAuthExt> {
    token_mngr: SingleThreadTokenManager<HTTPClientT>,
    receiver: Receiver<WorkerCommand>,
}

impl<HTTPClientT: ClientAuthExt> Worker<HTTPClientT> {
    async fn run(mut self) {
        while let Some(cmd) = self.receiver.recv().await {
            match cmd {
                WorkerCommand::GetToken { respond_to } => {
                    let token = self.token_mngr.get_token().await;
                    respond_to.send(Ok(token)).unwrap();
                }
                WorkerCommand::RefreshToken { respond_to } => {
                    let result = self.token_mngr.refresh_token().await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to refresh token: {:?}", e);
                    }
                }
            }
        }
    }
}

pub struct TokenManager {
    sender: Sender<WorkerCommand>,
}

impl TokenManager {
    pub fn spawn<HTTPClientT: ClientAuthExt + 'static>(
        client_id: String,
        http_client: Arc<HTTPClientT>,
        token_file: CachedFile<Token>,
        private_key_file: File,
        refresh_cooldown: Duration,
    ) -> Self {
        let (sender, receiver) = mpsc::channel(64);
        let worker = Worker {
            token_mngr: SingleThreadTokenManager::new(
                client_id,
                http_client,
                token_file,
                private_key_file,
                refresh_cooldown,
            ),
            receiver,
        };
        tokio::spawn(worker.run());
        Self { sender }
    }

    pub async fn get_token(&self) -> Result<Arc<Token>, AuthErr> {
        let (send, recv) = oneshot::channel();
        self.sender.send(WorkerCommand::GetToken { respond_to: send }).await.map_err(|e| AuthErr::SendActorMessageErr(SendActorMessageErr{
            source: Box::new(e),
            trace: trace!(),
        }))?;
        recv.await.map_err(|e| AuthErr::ReceiveActorMessageErr(ReceiveActorMessageErr{
            source: Box::new(e),
            trace: trace!(),
        }))?
    }

    pub async fn refresh_token(&self) -> Result<(), AuthErr> {
        let (send, recv) = oneshot::channel();
        self.sender.send(WorkerCommand::RefreshToken { respond_to: send }).await.map_err(|e| AuthErr::SendActorMessageErr(SendActorMessageErr{
            source: Box::new(e),
            trace: trace!(),
        }))?;
        recv.await.map_err(|e| AuthErr::ReceiveActorMessageErr(ReceiveActorMessageErr{
            source: Box::new(e),
            trace: trace!(),
        }))?
    }
}