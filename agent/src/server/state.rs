// standard library
use std::sync::Arc;

// internal crates
use crate::auth::token_mngr::TokenManager;
use crate::crypt::jwt;
use crate::filesys::{cached_file::CachedFile, file::File};
use crate::http::client::HTTPClient;
use crate::server::errors::{
    ServerErr,
    ServerAuthErr,
    ServerCryptErr,
    ServerFileSysErr,
};
use crate::storage::agent::Agent;
use crate::storage::concrete_configs::ConcreteConfigCache;
use crate::storage::digests::ConfigSchemaDigestCache;
use crate::storage::layout::StorageLayout;
use crate::storage::token::Token;
use crate::trace;

// external crates
use chrono::Duration;
use tracing::error;


type ClientID = String;

#[derive(Clone)]
pub struct ServerState {
    pub http_client: Arc<HTTPClient>,
    pub config_schema_digest_cache: Arc<ConfigSchemaDigestCache>,
    pub concrete_config_cache: Arc<ConcreteConfigCache>,
    pub token_mngr: Arc<TokenManager>,
}

pub async fn init_state(layout: StorageLayout) -> Result<ServerState, ServerErr> {
    // storage layout stuff
    let auth_dir = layout.auth_dir();
    let agent_file = layout.agent_file();
    let private_key_file = auth_dir.private_key_file();
    let token_file = CachedFile::new_with_default(
        auth_dir.token_file(),
        Token::default(),
    ).await.map_err(|e| ServerErr::FileSysErr(ServerFileSysErr {
        source: e,
        trace: trace!(),
    }))?;

    let client_id = get_client_id(&agent_file, &token_file).await?;

    // initialize the http client
    let http_client = Arc::new(HTTPClient::new().await);

    // initialize the token manager
    let token_mngr = TokenManager::spawn(
        client_id,
        http_client.clone(),
        token_file,
        private_key_file,
        Duration::seconds(15),
    ).map_err(|e| ServerErr::AuthErr(ServerAuthErr {
        source: e,
        trace: trace!(),
    }))?;

    // initialize the server state
    let server_state = ServerState {
        http_client,
        config_schema_digest_cache: Arc::new(ConfigSchemaDigestCache::spawn(
            layout.config_schema_digest_cache(),
        )),
        concrete_config_cache: Arc::new(ConcreteConfigCache::spawn(
            layout.concrete_config_cache(),
        )),
        token_mngr: Arc::new(token_mngr),
    };

    Ok(server_state)
}

pub async fn get_client_id(
    agent_file: &File,
    token_file: &CachedFile<Token>,
) -> Result<ClientID, ServerErr> {

    // attempt to get the client id from the agent file
    match agent_file.read_json::<Agent>().await {
        Ok(agent) => {
            return Ok(agent.client_id);
        }
        Err(e) => {
            error!("Error reading agent file: {:?}", e);
        }
    }

    // attempt to get the client id from the existing token on file
    let token = token_file.read();
    let client_id = jwt::extract_client_id(&token.token).map_err(|e| ServerErr::CryptErr(ServerCryptErr {
        source: e,
        trace: trace!(),
    }))?;

    // write the client id to the agent file
    let agent = Agent { client_id: client_id.clone() };
    agent_file.write_json(&agent, true, true).await.map_err(|e| ServerErr::FileSysErr(ServerFileSysErr {
        source: e,
        trace: trace!(),
    }))?;

    Ok(client_id)
}