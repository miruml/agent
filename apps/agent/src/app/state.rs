// standard library
use std::future::Future;
use std::sync::Arc;

// internal crates
use crate::activity::ActivityTracker;
use crate::auth::{
    token_mngr::{TokenManager, TokenManagerExt},
    token::Token,
};
use crate::crypt::jwt;
use crate::deploy::fsm;
use crate::filesys::{cached_file::CachedFile, file::File, path::PathExt};
use crate::http::client::HTTPClient;
use crate::models::agent::Agent;
use crate::server::errors::*;
use crate::storage::{
    caches::{CacheCapacities, Caches},
    layout::StorageLayout,
};
use crate::sync::syncer::{Syncer, SyncerArgs};
use crate::trace;

pub type DeviceID = String;

#[derive(Clone, Debug)]
pub struct AppState {
    pub device_id: String,
    pub http_client: Arc<HTTPClient>,
    pub syncer: Arc<Syncer>,
    pub caches: Arc<Caches>,
    pub token_mngr: Arc<TokenManager>,
    pub activity_tracker: Arc<ActivityTracker>,
}

impl AppState {
    pub async fn init(
        layout: &StorageLayout,
        cache_capacities: CacheCapacities,
        http_client: Arc<HTTPClient>,
        fsm_settings: fsm::Settings,
    ) -> Result<(Self, impl Future<Output = ()>), ServerErr> {

        // storage layout stuff
        let auth_dir = layout.auth_dir();
        let private_key_file = auth_dir.private_key_file();
        private_key_file.assert_exists().map_err(|e| {
            ServerErr::FileSysErr(Box::new(ServerFileSysErr {
                source: e,
                trace: trace!(),
            }))
        })?;
        let agent_file = layout.agent_file();
        let token_file = CachedFile::new_with_default(auth_dir.token_file(), Token::default())
            .await
            .map_err(|e| {
                ServerErr::FileSysErr(Box::new(ServerFileSysErr {
                    source: e,
                    trace: trace!(),
                }))
            })?;

        // get the device id
        let device_id = Self::init_device_id(&agent_file, &token_file).await?;

        // initialize the caches
        let (caches, caches_shutdown_handle) = Caches::init(layout, cache_capacities).await.map_err(|e| {
            ServerErr::StorageErr(Box::new(ServerStorageErr {
                source: e,
                trace: trace!(),
            }))
        })?;
        let caches = Arc::new(caches);

        // initialize the token manager
        let (token_mngr, token_mngr_handle) = TokenManager::spawn(
            64,
            device_id.clone(),
            http_client.clone(),
            token_file,
            private_key_file,
        )
        .map_err(|e| {
            ServerErr::AuthErr(Box::new(ServerAuthErr {
                source: e,
                trace: trace!(),
            }))
        })?;
        let token_mngr = Arc::new(token_mngr);

        // initialize the syncer
        let (syncer, syncer_handle) = Syncer::spawn(
            64,
            SyncerArgs {
                device_id: device_id.clone(),
                http_client: http_client.clone(),
                token_mngr: token_mngr.clone(),
                cfg_inst_cache: caches.cfg_inst_metadata.clone(),
                cfg_inst_data_cache: caches.cfg_inst_data.clone(),
                deployment_dir: layout.config_instance_deployment_dir(),
                fsm_settings,
            },
        )
        .map_err(|e| {
            ServerErr::SyncErr(Box::new(ServerSyncErr {
                source: e,
                trace: trace!(),
            }))
        })?;
        let syncer = Arc::new(syncer);

        // initialize the activity tracker
        let activity_tracker = Arc::new(ActivityTracker::new());

        let shutdown_handle = async move {
            let handles = vec![
                token_mngr_handle,
                syncer_handle,
            ];

            futures::future::join(
                futures::future::join_all(handles),
                caches_shutdown_handle,
            ).await;
        };

        Ok((AppState {
            device_id,
            http_client,
            syncer,
            caches,
            token_mngr,
            activity_tracker,
        }, shutdown_handle))
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
                return Err(ServerErr::MissingDeviceIDErr(Box::new(
                    MissingDeviceIDErr {
                        agent_file_err,
                        jwt_err: e,
                        trace: trace!(),
                    },
                )));
            }
        };

        Ok(device_id)
    }

    pub async fn shutdown(&self) -> Result<(), ServerErr> {
        // shutdown the syncer
        self.syncer.shutdown().await.map_err(|e| {
            ServerErr::SyncErr(Box::new(ServerSyncErr {
                source: e,
                trace: trace!(),
            }))
        })?;

        // shutdown the caches
        self.caches.shutdown().await.map_err(|e| {
            ServerErr::StorageErr(Box::new(ServerStorageErr {
                source: e,
                trace: trace!(),
            }))
        })?;

        // shutdown the token manager
        self.token_mngr.shutdown().await.map_err(|e| {
            ServerErr::AuthErr(Box::new(ServerAuthErr {
                source: e,
                trace: trace!(),
            }))
        })?;

        Ok(())
    }
}
