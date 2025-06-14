// standard library
use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

// internal crates
use crate::auth::token_mngr::TokenManager;
use crate::crypt::jwt;
use crate::filesys::{cached_file::CachedFile, file::File, path::PathExt};
use crate::http::client::HTTPClient;
use crate::server::errors::{
    MissingDeviceIDErr, ServerAuthErr, ServerErr, ServerFileSysErr, ServerStorageErr,
};
use crate::storage::agent::Agent;
use crate::storage::config_instances::ConfigInstanceCache;
use crate::storage::digests::ConfigSchemaDigestCache;
use crate::storage::layout::StorageLayout;
use crate::storage::token::Token;
use crate::trace;

type DeviceID = String;

#[derive(Clone, Debug)]
pub struct ServerState {
    pub device_id: DeviceID,
    pub http_client: Arc<HTTPClient>,
    pub config_schema_digest_cache: Arc<ConfigSchemaDigestCache>,
    pub config_instance_cache: Arc<ConfigInstanceCache>,
    pub token_mngr: Arc<TokenManager>,
    pub last_activity: Arc<AtomicU64>,
}

impl ServerState {
    pub async fn new(
        layout: StorageLayout,
        http_client: Arc<HTTPClient>,
    ) -> Result<(Self, impl Future<Output = ()>), ServerErr> {
        // storage layout stuff
        let auth_dir = layout.auth_dir();
        let private_key_file = auth_dir.private_key_file();
        private_key_file.assert_exists().map_err(|e| {
            ServerErr::FileSysErr(ServerFileSysErr {
                source: Box::new(e),
                trace: trace!(),
            })
        })?;
        let agent_file = layout.agent_file();
        let token_file = CachedFile::new_with_default(auth_dir.token_file(), Token::default())
            .await
            .map_err(|e| {
                ServerErr::FileSysErr(ServerFileSysErr {
                    source: Box::new(e),
                    trace: trace!(),
                })
            })?;

        // get the device id
        let device_id = Self::init_device_id(&agent_file, &token_file).await?;

        // initialize the caches
        let (config_schema_digest_cache, config_schema_digest_cache_handle) =
            ConfigSchemaDigestCache::spawn(layout.config_schema_digest_cache());
        let config_schema_digest_cache = Arc::new(config_schema_digest_cache);
        let (config_instance_cache, config_instance_cache_handle) =
            ConfigInstanceCache::spawn(layout.config_instance_cache());
        let config_instance_cache = Arc::new(config_instance_cache);

        // initialize the token manager
        let (token_mngr, token_mngr_handle) = TokenManager::spawn(
            device_id.clone(),
            http_client.clone(),
            token_file,
            private_key_file,
        )
        .map_err(|e| {
            ServerErr::AuthErr(ServerAuthErr {
                source: Box::new(e),
                trace: trace!(),
            })
        })?;
        let token_mngr = Arc::new(token_mngr);

        // initialize the server state
        let server_state = ServerState {
            device_id,
            http_client,
            config_schema_digest_cache,
            config_instance_cache,
            token_mngr,
            last_activity: Arc::new(AtomicU64::new(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            )),
        };

        // return the shutdown handler
        let shutdown_handle = async move {
            let handles = vec![
                config_schema_digest_cache_handle,
                config_instance_cache_handle,
                token_mngr_handle,
            ];

            futures::future::join_all(handles).await;
        };

        Ok((server_state, shutdown_handle))
    }

    pub async fn shutdown(&self) -> Result<(), ServerErr> {
        // shutdown the caches
        self.config_schema_digest_cache
            .shutdown()
            .await
            .map_err(|e| {
                ServerErr::StorageErr(ServerStorageErr {
                    source: Box::new(e),
                    trace: trace!(),
                })
            })?;
        self.config_instance_cache.shutdown().await.map_err(|e| {
            ServerErr::StorageErr(ServerStorageErr {
                source: Box::new(e),
                trace: trace!(),
            })
        })?;

        // shutdown the token manager
        self.token_mngr.shutdown().await.map_err(|e| {
            ServerErr::AuthErr(ServerAuthErr {
                source: Box::new(e),
                trace: trace!(),
            })
        })?;

        Ok(())
    }

    async fn init_device_id(
        agent_file: &File,
        token_file: &CachedFile<Token>,
    ) -> Result<DeviceID, ServerErr> {
        // attempt to get the device id from the agent file
        let agent_file_err = match agent_file.read_json::<Agent>().await {
            Ok(agent) => {
                return Ok(agent.device_id);
            }
            Err(e) => e,
        };

        // attempt to get the device id from the existing token on file
        let token = token_file.read();
        let device_id = match jwt::extract_device_id(&token.token) {
            Ok(device_id) => device_id,
            Err(e) => {
                return Err(ServerErr::MissingDeviceIDErr(MissingDeviceIDErr {
                    agent_file_err: Box::new(agent_file_err),
                    jwt_err: Box::new(e),
                    trace: trace!(),
                }));
            }
        };

        Ok(device_id)
    }

    pub fn record_activity(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.last_activity.store(now, Ordering::Relaxed);
    }
}
