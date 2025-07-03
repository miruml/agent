// standard crates
use std::sync::Arc;

// internal crates
use crate::auth::token_mngr::TokenManager;
use crate::crud::prelude::*;
use crate::deploy::{apply::apply, fsm};
use crate::filesys::dir::Dir;
use crate::http::{client::HTTPClient, config_instances::ConfigInstancesExt};
use crate::storage::config_instances::{ConfigInstanceCache, ConfigInstanceDataCache};
use crate::sync::errors::{
    ReceiveActorMessageErr, SendActorMessageErr, SyncAuthErr, SyncCrudErr, SyncDeployErr, SyncErr,
};
use crate::sync::pull::pull_config_instances;
use crate::sync::push::push_config_instances;
use crate::trace;

// external crates
use chrono::{DateTime, TimeDelta, Utc};
use tokio::sync::{
    mpsc,
    mpsc::{Receiver, Sender},
    oneshot,
};
use tokio::task::JoinHandle;
use tracing::{error, info};

// ======================== SINGLE-THREADED IMPLEMENTATION ========================= //
pub struct SingleThreadSyncer<HTTPClientT: ConfigInstancesExt> {
    device_id: String,
    http_client: Arc<HTTPClientT>,
    token_mngr: Arc<TokenManager>,
    cfg_inst_cache: Arc<ConfigInstanceCache>,
    cfg_inst_data_cache: Arc<ConfigInstanceDataCache>,
    deployment_dir: Dir,
    fsm_settings: fsm::Settings,
    last_synced_at: DateTime<Utc>,
}

impl<HTTPClientT: ConfigInstancesExt> SingleThreadSyncer<HTTPClientT> {
    pub fn new(
        device_id: String,
        http_client: Arc<HTTPClientT>,
        token_mngr: Arc<TokenManager>,
        cfg_inst_cache: Arc<ConfigInstanceCache>,
        cfg_inst_data_cache: Arc<ConfigInstanceDataCache>,
        deployment_dir: Dir,
        fsm_settings: fsm::Settings,
    ) -> Self {
        Self {
            device_id,
            http_client,
            token_mngr,
            cfg_inst_cache,
            cfg_inst_data_cache,
            deployment_dir,
            fsm_settings,
            last_synced_at: DateTime::<Utc>::UNIX_EPOCH,
        }
    }

    fn get_last_synced_at(&self) -> DateTime<Utc> {
        self.last_synced_at
    }

    async fn sync(&mut self, cooldown: TimeDelta) -> Result<(), SyncErr> {
        if cooldown > Utc::now() - self.last_synced_at {
            return Ok(());
        }

        self.last_synced_at = Utc::now();

        let token = self.token_mngr.get_token().await.map_err(|e| {
            SyncErr::AuthErr(Box::new(SyncAuthErr {
                source: e,
                trace: trace!(),
            }))
        })?;

        let mut errors = Vec::new();

        // pull config instances from server
        let result = pull_config_instances(
            self.cfg_inst_cache.as_ref(),
            self.cfg_inst_data_cache.as_ref(),
            self.http_client.as_ref(),
            &self.device_id,
            &token.token,
        )
        .await;
        match result {
            Ok(_) => (),
            Err(e) => {
                errors.push(e);
            }
        };

        // read the config instances which need to be applied
        let cfg_insts_to_apply = self.cfg_inst_cache
            .find_where(|instance| fsm::is_action_required(fsm::next_action(instance, true)))
            .await
            .map_err(|e| {
                SyncErr::CrudErr(Box::new(SyncCrudErr {
                    source: e,
                    trace: trace!(),
                }))
            })?;
        let cfg_insts_to_apply = cfg_insts_to_apply
            .into_iter()
            .map(|instance| (instance.id.clone(), instance))
            .collect();

        // apply deployments
        apply(
            cfg_insts_to_apply,
            self.cfg_inst_cache.as_ref(),
            self.cfg_inst_data_cache.as_ref(),
            &self.deployment_dir,
            &self.fsm_settings,
        )
        .await
        .map_err(|e| {
            SyncErr::DeployErr(Box::new(SyncDeployErr {
                source: e,
                trace: trace!(),
            }))
        })?;

        // push config instances to server
        let result =
            push_config_instances(
                self.cfg_inst_cache.as_ref(),
                self.http_client.as_ref(),
                &token.token,
            )
            .await;
        match result {
            Ok(_) => (),
            Err(e) => {
                errors.push(e);
            }
        };

        Ok(())
    }
}

// ========================= MULTI-THREADED IMPLEMENTATION ========================= //
pub enum WorkerCommand {
    Shutdown {
        respond_to: oneshot::Sender<Result<(), SyncErr>>,
    },
    Sync {
        respond_to: oneshot::Sender<Result<(), SyncErr>>,
        cooldown: TimeDelta,
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
    pub fn new(syncer: SingleThreadSyncer<HTTPClientT>, receiver: Receiver<WorkerCommand>) -> Self {
        Self { syncer, receiver }
    }
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
                    cooldown,
                } => {
                    let result = self.syncer.sync(
                        cooldown,
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

#[derive(Debug)]
pub struct Syncer {
    sender: Sender<WorkerCommand>,
}

impl Syncer {
    pub fn spawn(
        buffer_size: usize,
        device_id: String,
        http_client: Arc<HTTPClient>,
        token_mngr: Arc<TokenManager>,
        cfg_inst_cache: Arc<ConfigInstanceCache>,
        cfg_inst_data_cache: Arc<ConfigInstanceDataCache>,
        deployment_dir: Dir,
        fsm_settings: fsm::Settings,
    ) -> Result<(Self, JoinHandle<()>), SyncErr> {
        let (sender, receiver) = mpsc::channel(buffer_size);
        let worker = Worker {
            syncer: SingleThreadSyncer::new(
                device_id,
                http_client,
                token_mngr,
                cfg_inst_cache,
                cfg_inst_data_cache,
                deployment_dir,
                fsm_settings,
            ),
            receiver,
        };
        let worker_handle = tokio::spawn(worker.run());
        Ok((Self { sender }, worker_handle))
    }

    pub fn new(sender: Sender<WorkerCommand>) -> Self {
        Self { sender }
    }

    pub async fn shutdown(&self) -> Result<(), SyncErr> {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::Shutdown { respond_to: send })
            .await
            .map_err(|e| {
                SyncErr::SendActorMessageErr(Box::new(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                }))
            })?;
        recv.await.map_err(|e| {
            SyncErr::ReceiveActorMessageErr(Box::new(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            }))
        })??;
        info!("Syncer shutdown complete");
        Ok(())
    }

    pub async fn sync(
        &self,
        cooldown: TimeDelta,
    ) -> Result<(), SyncErr> {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::Sync {
                respond_to: send,
                cooldown,
            })
            .await
            .map_err(|e| {
                SyncErr::SendActorMessageErr(Box::new(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                }))
            })?;
        recv.await.map_err(|e| {
            SyncErr::ReceiveActorMessageErr(Box::new(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            }))
        })?
    }

    pub async fn get_last_synced_at(&self) -> Result<DateTime<Utc>, SyncErr> {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::GetLastSyncedAt { respond_to: send })
            .await
            .map_err(|e| {
                SyncErr::SendActorMessageErr(Box::new(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                }))
            })?;
        recv.await.map_err(|e| {
            SyncErr::ReceiveActorMessageErr(Box::new(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            }))
        })?
    }
}
