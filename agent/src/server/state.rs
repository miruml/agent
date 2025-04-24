// standard library
use std::sync::Arc;

// internal crates
use crate::http::client::HTTPClient;
use crate::server::errors::{
    ServerErr,
    ServerFileSysErr,
};
use crate::filesys::cached_file::CachedFile;
use crate::storage::concrete_configs::ConcreteConfigCache;
use crate::storage::digests::ConfigSchemaDigestCache;
use crate::storage::layout::StorageLayout;
use crate::storage::token::Token;
use crate::trace;

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