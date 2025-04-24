// standard library
use std::sync::Arc;

// internal crates
use crate::auth::issue_token;
use crate::filesys::file::File;
use crate::http::{
    auth::ClientAuthExt,
    client::HTTPClient,
};
use crate::server::errors::{
    ServerErr,
    ServerAuthErr,
    ServerFileSysErr,
    ServerHTTPErr,
    TimestampConversionErr,
};
use crate::auth::token::Token;
use crate::filesys::cached_file::CachedFile;
use crate::storage::concrete_configs::ConcreteConfigCache;
use crate::storage::digests::ConfigSchemaDigestCache;
use crate::storage::layout::StorageLayout;
use crate::trace;
use openapi_client::models::{
    IssueClientClaims, IssueClientTokenRequest,
};

// external crates
use chrono::{Utc, TimeZone, DateTime};


#[derive(Clone)]
pub struct ServerState {
    pub http_client: Arc<HTTPClient>,
    pub config_schema_digest_cache: Arc<ConfigSchemaDigestCache>,
    pub concrete_config_cache: Arc<ConcreteConfigCache>,
    pub token_file: Arc<CachedFile<Token>>,
}

pub async fn init_state(layout: StorageLayout) -> Result<ServerState, ServerErr> {
    let auth_dir = layout.auth_dir();
    let token_file = CachedFile::new_with_default(
        auth_dir.token_file(),
        Token::default(),
    ).await.map_err(|e| ServerErr::FileSysErr(ServerFileSysErr {
        source: e,
        trace: trace!(),
    }))?;

    let server_state = ServerState {
        http_client: Arc::new(HTTPClient::new().await),
        config_schema_digest_cache: Arc::new(ConfigSchemaDigestCache::spawn(
            layout.config_schema_digest_cache(),
        )),
        concrete_config_cache: Arc::new(ConcreteConfigCache::spawn(
            layout.concrete_config_cache(),
        )),
        token_file: Arc::new(token_file),
    };

    Ok(server_state)
}

pub async fn refresh_token<HTTPClientT: ClientAuthExt>(
    http_client: &HTTPClientT,
    token_file: &mut CachedFile<Token>,
    private_key_file: &File,
    client_id: &str,
) -> Result<(), ServerErr> {
    // issue a new token
    let token = issue_token(http_client, private_key_file, client_id).await?;

    // update the token file
    token_file.write(token).await.map_err(|e| ServerErr::FileSysErr(ServerFileSysErr {
        source: e,
        trace: trace!(),
    }))?;

    Ok(())
}

pub async fn issue_token<HTTPClientT: ClientAuthExt>(
    http_client: &HTTPClientT,
    private_key_file: &File,
    client_id: &str,
) -> Result<Token, ServerErr> {
    // prepare the token request
    let token_request = issue_token::prepare_issue_token_request(
        client_id,
        private_key_file,
    ).await.map_err(|e| ServerErr::AuthErr(ServerAuthErr {
        source: e,
        trace: trace!(),
    }))?;

    // convert it to the http client format
    let expiration = match Utc.timestamp_opt(
        token_request.claims.expiration,
        0
    ).single() {
        Some(expiration) => expiration,
        None => return Err(ServerErr::TimestampConversionErr(TimestampConversionErr {
            msg: format!("unable to convert unix timestamp '{}' to utc", token_request.claims.expiration),
            trace: trace!(),
        })),
    };
    let claims = IssueClientClaims {
        client_id: client_id.to_string(),
        nonce: token_request.claims.nonce,
        expiration: expiration.to_rfc3339(),
    };
    let payload = IssueClientTokenRequest {
        claims: Box::new(claims),
        signature: token_request.signature,
    };

    // send the token request
    let resp = http_client.issue_client_token(
        client_id,
        &payload,
    ).await.map_err(|e| ServerErr::HTTPErr(ServerHTTPErr {
        source: e,
        trace: trace!(),
    }))?;


    // format the response
    let expires_at = resp.expires_at.parse::<DateTime<Utc>>().map_err(|e| ServerErr::TimestampConversionErr(TimestampConversionErr {
            msg: format!("failed to parse date time '{}' from string: {}", resp.expires_at, e),
            trace: trace!(),
        }))?;
    Ok(Token {
        token: resp.token,
        expires_at,
    })
}

