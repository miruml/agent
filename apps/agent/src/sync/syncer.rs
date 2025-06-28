// standard crates
use std::sync::Arc;

// internal crates
use crate::auth::token_mngr::TokenManager;
use crate::crud::prelude::*;
use crate::deploy::{
    apply::apply_deployments,
    errors::{DeployErr, DeployCacheErr},
    fsm,
    observer::Observer,
};
use crate::filesys::dir::Dir;
use crate::http::config_instances::ConfigInstancesExt;
use crate::models::config_instance::ConfigInstance;
use crate::storage::config_instances::{
    ConfigInstanceCache,
    ConfigInstanceCacheEntry,
    ConfigInstanceDataCache,
};
use crate::sync::pull::pull_config_instances;
use crate::sync::push::push_config_instances;
use crate::sync::errors::{
    SyncErr,
    SyncCrudErr,
    SyncDeployErr,
    SendActorMessageErr,
    ReceiveActorMessageErr,
    SyncAuthErr,
};
use crate::trace;

// external crates
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tracing::{error, info};


pub struct StorageObserver<'a> {
    pub cfg_inst_cache: &'a ConfigInstanceCache,
}

#[async_trait]
impl<'a> Observer for StorageObserver<'a> {
    async fn on_update(&mut self, instance: &ConfigInstance) -> Result<(), DeployErr> {
        let overwrite = true;
        self.cfg_inst_cache.write(
            instance.id.clone(),
            instance.clone(),
            is_dirty,
            overwrite,
        ).await.map_err(|e| {
            DeployErr::CacheErr(DeployCacheErr {
                source: e,
                trace: trace!(),
            })
        })
    }
}

fn is_dirty(old: Option<&ConfigInstanceCacheEntry>, new: &ConfigInstance) -> bool {
    let old = match old {
        Some(old) => old,
        None => return true,
    };
    old.is_dirty ||
    old.value.activity_status != new.activity_status || 
    old.value.error_status != new.error_status
}


// ======================== SINGLE-THREADED IMPLEMENTATION ========================= //
pub struct SingleThreadSyncer<HTTPClientT: ConfigInstancesExt> {
    device_id: String,
    http_client: Arc<HTTPClientT>,
    token_mngr: Arc<TokenManager>,
    deployment_dir: Dir,
    fsm_settings: fsm::Settings,
    last_synced_at: DateTime<Utc>,
}

impl<HTTPClientT: ConfigInstancesExt> SingleThreadSyncer<HTTPClientT> {
    
    pub fn new(
        device_id: String,
        http_client: Arc<HTTPClientT>,
        token_mngr: Arc<TokenManager>,
        deployment_dir: Dir,
        fsm_settings: fsm::Settings,
    ) -> Self {
        Self {
            device_id,
            http_client,
            token_mngr,
            deployment_dir,
            fsm_settings,
            last_synced_at: DateTime::<Utc>::UNIX_EPOCH,
        }
    }

    pub fn get_last_synced_at(&self) -> DateTime<Utc> {
        self.last_synced_at
    }

    pub async fn sync(
        &mut self,
        cfg_inst_cache: &ConfigInstanceCache,
        cfg_inst_data_cache: &ConfigInstanceDataCache,
    ) -> Result<(), SyncErr> {

        self.last_synced_at = Utc::now();

        let token = self.token_mngr.get_token().await.map_err(|e| SyncErr::AuthErr(SyncAuthErr {
            source: e,
            trace: trace!(),
        }))?;

        // pull config instances from server
        pull_config_instances(
            cfg_inst_cache,
            cfg_inst_data_cache,
            self.http_client.as_ref(),
            &self.device_id,
            &token.token,
        ).await?;

        // read the config instances which need to be applied
        let cfg_insts_to_apply = cfg_inst_cache.find_where(
            |instance| { fsm::is_action_required(instance) }
        ).await.map_err(|e| SyncErr::CrudErr(SyncCrudErr {
            source: e,
            trace: trace!(),
        }))?;
        let cfg_insts_to_apply = cfg_insts_to_apply.into_iter().map(|instance| (instance.id.clone(), instance)).collect();

        // observers
        let mut observers: Vec<&mut dyn Observer> = Vec::new();
        let mut storage_observer = StorageObserver { cfg_inst_cache };
        observers.push(&mut storage_observer);

        // apply deployments
        apply_deployments(
            cfg_insts_to_apply,
            cfg_inst_cache,
            cfg_inst_data_cache,
            &self.deployment_dir,
            &self.fsm_settings,
            &mut observers[..],
        ).await.map_err(|e| SyncErr::DeployErr(SyncDeployErr {
            source: e,
            trace: trace!(),
        }))?;

        // push config instances to server
        push_config_instances(
            cfg_inst_cache,
            self.http_client.as_ref(),
            &token.token,
        ).await?;

        Ok(())
    }
}



// ========================= MULTI-THREADED IMPLEMENTATION ========================= //
enum WorkerCommand {
    Shutdown {
        respond_to: oneshot::Sender<Result<(), SyncErr>>,
    },
    Sync {
        respond_to: oneshot::Sender<Result<(), SyncErr>>,
        cfg_inst_cache: Arc<ConfigInstanceCache>,
        cfg_inst_data_cache: Arc<ConfigInstanceDataCache>,
    },
    GetLastSyncedAt {
        respond_to: oneshot::Sender<Result<DateTime<Utc>, SyncErr>>,
    },
}

pub struct Worker<HTTPClientT: ConfigInstancesExt + Send> {
    syncer: SingleThreadSyncer<HTTPClientT>,
    receiver: Receiver<WorkerCommand>,
}

impl<HTTPClientT: ConfigInstancesExt + Send> Worker<HTTPClientT> {
    pub async fn run(mut self) {
        while let Some(cmd) = self.receiver.recv().await {
            match cmd {
                WorkerCommand::Shutdown { respond_to } => {
                    if let Err(e) = respond_to.send(Ok(())) {
                        error!("Actor failed to send shutdown response: {:?}", e);
                    }
                    break;
                }
                WorkerCommand::Sync { 
                    respond_to,
                    cfg_inst_cache,
                    cfg_inst_data_cache,
                } => {
                    let result = self.syncer.sync(
                        cfg_inst_cache.as_ref(),
                        cfg_inst_data_cache.as_ref(),
                    ).await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to send sync response: {:?}", e);
                    }
                }
                WorkerCommand::GetLastSyncedAt { respond_to } => {
                    let result = self.syncer.get_last_synced_at();
                    if let Err(e) = respond_to.send(Ok(result)) {
                        error!("Actor failed to send last synced at response: {:?}", e);
                    }
                }
            }
        }
    }
}

pub struct Syncer {
    sender: Sender<WorkerCommand>,
}

impl Syncer {
    pub fn spawn<HTTPClientT: ConfigInstancesExt + 'static>(
        device_id: String,
        http_client: Arc<HTTPClientT>,
        token_mngr: Arc<TokenManager>,
        deployment_dir: Dir,
        fsm_settings: fsm::Settings,
    ) -> (Self, JoinHandle<()>) {
        let (sender, receiver) = mpsc::channel(100);
        let worker = Worker {
            syncer: SingleThreadSyncer::new(
                device_id,
                http_client,
                token_mngr,
                deployment_dir,
                fsm_settings,
            ),
            receiver,
        };
        let worker_handle = tokio::spawn( worker.run() );
        (Self { sender }, worker_handle)
    }

    pub async fn shutdown(&self) -> Result<(), SyncErr> {
        let (send, recv) = oneshot::channel();
        self.sender.send(WorkerCommand::Shutdown { respond_to: send }).await.map_err(|e| {
            SyncErr::SendActorMessageErr(SendActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            })
        })?;
        recv.await.map_err(|e| {
            SyncErr::ReceiveActorMessageErr(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            })
        })??;
        info!("Syncer shutdown complete");
        Ok(())
    }

    pub async fn sync(
        &self,
        cfg_inst_cache: Arc<ConfigInstanceCache>,
        cfg_inst_data_cache: Arc<ConfigInstanceDataCache>,
    ) -> Result<(), SyncErr> {
        let (send, recv) = oneshot::channel();
        self.sender.send(WorkerCommand::Sync { 
            respond_to: send,
            cfg_inst_cache,
            cfg_inst_data_cache,
        }).await.map_err(|e| {
            SyncErr::SendActorMessageErr(SendActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            })
        })?;
        recv.await.map_err(|e| {
            SyncErr::ReceiveActorMessageErr(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            })
        })?
    }

    pub async fn get_last_synced_at(&self) -> Result<DateTime<Utc>, SyncErr> {
        let (send, recv) = oneshot::channel();
        self.sender.send(WorkerCommand::GetLastSyncedAt { respond_to: send }).await.map_err(|e| {
            SyncErr::SendActorMessageErr(SendActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            })
        })?;
        recv.await.map_err(|e| {
            SyncErr::ReceiveActorMessageErr(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            })
        })?
    }
}



