// standard crates
use std::sync::Arc;
use std::time::Duration;

// internal crates
use crate::auth::token_mngr::{TokenManager, TokenManagerExt};
use crate::crud::prelude::*;
use crate::deploy::{apply::apply, fsm};
use crate::errors::*;
use crate::filesys::dir::Dir;
use crate::http::{client::HTTPClient, config_instances::ConfigInstancesExt};
use crate::storage::config_instances::{ConfigInstanceCache, ConfigInstanceDataCache};
use crate::sync::errors::*;
use crate::sync::pull::pull_config_instances;
use crate::sync::push::push_config_instances;
use crate::trace;
use crate::utils::{calc_exp_backoff, CooldownOptions};

// external crates
use chrono::{DateTime, TimeDelta, Utc};
use tokio::sync::{
    mpsc,
    oneshot,
    watch,
};
use tokio::task::JoinHandle;
use tracing::{debug, error, info};


// =============================== SYNCER EVENTS ================================== //
#[derive(Debug, Clone, PartialEq)]
pub struct SyncFailure {
    pub is_network_connection_error: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CooldownEnd {
    FromSyncSuccess,
    FromSyncFailure,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SyncEvent {
    SyncSuccess,
    SyncFailed(SyncFailure),
    CooldownEnd(CooldownEnd),
}

// ======================== SINGLE-THREADED IMPLEMENTATION ========================= //
pub struct SyncerArgs<
    HTTPClientT: ConfigInstancesExt,
    TokenManagerT: TokenManagerExt,
> {
    pub device_id: String,
    pub http_client: Arc<HTTPClientT>,
    pub token_mngr: Arc<TokenManagerT>,
    pub cfg_inst_cache: Arc<ConfigInstanceCache>,
    pub cfg_inst_data_cache: Arc<ConfigInstanceDataCache>,
    pub deployment_dir: Dir,
    pub fsm_settings: fsm::Settings,
    pub cooldown_options: CooldownOptions,
}

#[derive(Debug, Clone)]
pub struct SyncState {
    pub last_sync_attempted_at: DateTime<Utc>,
    pub last_successful_sync_at: DateTime<Utc>,
    pub cooldown_ends_at: DateTime<Utc>,
    pub err_streak: u32,
}

impl SyncState {
    pub fn is_in_cooldown(&self) -> bool {
        Utc::now() < self.cooldown_ends_at
    }
}

pub struct SingleThreadSyncer<HTTPClientT: ConfigInstancesExt> {
    device_id: String,
    http_client: Arc<HTTPClientT>,
    token_mngr: Arc<TokenManager>,
    cfg_inst_cache: Arc<ConfigInstanceCache>,
    cfg_inst_data_cache: Arc<ConfigInstanceDataCache>,
    deployment_dir: Dir,
    fsm_settings: fsm::Settings,

    // subscribers
    subscriber_tx: watch::Sender<SyncEvent>,
    subscriber_rx: watch::Receiver<SyncEvent>,

    // syncer state
    cooldown_options: CooldownOptions,
    state: SyncState,
}

impl<HTTPClientT: ConfigInstancesExt> SingleThreadSyncer<HTTPClientT> {
    pub fn new(
        args: SyncerArgs<HTTPClientT, TokenManager>,
    ) -> Self {
        let (subscriber_tx, subscriber_rx) = watch::channel(SyncEvent::SyncSuccess);
        Self {
            device_id: args.device_id,
            http_client: args.http_client,
            token_mngr: args.token_mngr,
            cfg_inst_cache: args.cfg_inst_cache,
            cfg_inst_data_cache: args.cfg_inst_data_cache,
            deployment_dir: args.deployment_dir,
            fsm_settings: args.fsm_settings,
            cooldown_options: args.cooldown_options,
            state: SyncState {
                last_sync_attempted_at: DateTime::<Utc>::UNIX_EPOCH,
                last_successful_sync_at: DateTime::<Utc>::UNIX_EPOCH,
                cooldown_ends_at: DateTime::<Utc>::UNIX_EPOCH,
                err_streak: 0,
            },
            subscriber_tx,
            subscriber_rx,
        }
    }

    fn subscribe(&self) -> Result<watch::Receiver<SyncEvent>, SyncErr> {
        Ok(self.subscriber_rx.clone())
    }

    fn schedule_cooldown_end_notification(
        &self,
        cooldown_end_at: DateTime<Utc>,
        is_success: bool,
    ) {
        if cooldown_end_at <= Utc::now() {
            return;
        }
        let cooldown_secs = cooldown_end_at.signed_duration_since(Utc::now()).num_seconds();
        let tx = self.subscriber_tx.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(cooldown_secs as u64)).await;
            let event = if is_success {
                SyncEvent::CooldownEnd(CooldownEnd::FromSyncSuccess)
            } else {
                SyncEvent::CooldownEnd(CooldownEnd::FromSyncFailure)
            };
            if let Err(e) = tx.send(event) {
                error!("failed to send cooldown ended event: {:?}", e);
            }
        });
    }

    async fn get_sync_state(&self) -> Result<SyncState, SyncErr> {
        Ok(self.state.clone())
    }

    #[cfg(feature = "test")]
    fn set_sync_state(&mut self, state: SyncState) {
        self.state = state;
    }

    async fn sync_if_not_in_cooldown(&mut self) -> Result<(), SyncErr> {
        if self.state.is_in_cooldown() {
            info!("skipping device sync since the cooldown ends at {:?} (err streak: {}, last successful sync at: {:?})",
                self.state.cooldown_ends_at,
                self.state.err_streak,
                self.state.last_successful_sync_at
            );
            return Ok(());
        }
        self.sync().await
    }

    async fn sync(&mut self) -> Result<(), SyncErr> {
        if self.state.is_in_cooldown() {
            return Err(SyncErr::InCooldownErr(Box::new(SyncerInCooldownErr {
                err_streak: self.state.err_streak,
                cooldown_ends_at: self.state.cooldown_ends_at,
                trace: trace!(),
            })));
        }

        self.state.last_sync_attempted_at = Utc::now();
        let result = self.sync_impl().await;
        let (success, cooldown_secs) = match &result {
            Ok(_) => {
                if let Err(e) = self.subscriber_tx.send(SyncEvent::SyncSuccess) {
                    error!("failed to send sync success event: {:?}", e);
                }
                if self.state.err_streak > 0 {
                    info!("successfully synced with backend after an error streak of {}", self.state.err_streak);
                } else {
                    info!("successfully synced with backend");
                }
                self.state.last_successful_sync_at = Utc::now();
                self.state.cooldown_ends_at = Utc::now();
                self.state.err_streak = 0;
                (true, self.cooldown_options.base_secs)
            }
            Err(e) => {
                if let Err(e) = self.subscriber_tx.send(SyncEvent::SyncFailed(SyncFailure {
                    is_network_connection_error: e.is_network_connection_error(),
                })) {
                    error!("failed to send sync failed event: {:?}", e);
                }
                // network connection errors are expected to happen and do not count
                // toward the error streak. We want to be able to retry syncing from network connection errors even if the previous errors were not network connection errors so we use an error streak of 0 when calculating the cooldown period
                if e.is_network_connection_error() {
                    debug!("unable to sync with backend due to a network connection error: {:?}", e);
                    (false, self.cooldown_options.base_secs)
                } else {
                    error!("unable to sync with backend: {:?}", e);
                    self.state.err_streak += 1;
                    (false, calc_exp_backoff(
                        self.cooldown_options.base_secs,
                        self.cooldown_options.growth_factor,
                        self.state.err_streak,
                        self.cooldown_options.max_secs,
                    ))
                }
            }
        };

        // schedule the cooldown end notification
        self.state.cooldown_ends_at = Utc::now() + TimeDelta::seconds(cooldown_secs);
        self.schedule_cooldown_end_notification(
            self.state.cooldown_ends_at,
            success,
        );
        info!("backend syncer cooling down for {cooldown_secs} seconds (until {:?})", self.state.cooldown_ends_at);

        result
    }

    async fn sync_impl(&mut self) -> Result<(), SyncErr> {
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

        if errors.is_empty() {
            Ok(())
        } else {
            Err(SyncErr::SyncErrors(Box::new(SyncErrors {
                source: errors,
                trace: trace!(),
            })))
        }
    }
}

// ========================= MULTI-THREADED IMPLEMENTATION ========================= //
#[allow(async_fn_in_trait)]
pub trait SyncerExt {
    async fn shutdown(&self) -> Result<(), SyncErr>;
    async fn get_sync_state(&self) -> Result<SyncState, SyncErr>;
    async fn is_in_cooldown(&self) -> Result<bool, SyncErr>;
    async fn get_cooldown_ends_at(&self) -> Result<DateTime<Utc>, SyncErr>;
    async fn sync(&self) -> Result<(), SyncErr>;
    async fn sync_if_not_in_cooldown(&self) -> Result<(), SyncErr>;
    async fn subscribe(&self) -> Result<watch::Receiver<SyncEvent>, SyncErr>;
}

pub enum WorkerCommand {
    Shutdown {
        respond_to: oneshot::Sender<Result<(), SyncErr>>,
    },
    GetSyncState {
        respond_to: oneshot::Sender<Result<SyncState, SyncErr>>,
    },
    #[cfg(feature = "test")]
    SetSyncState {
        state: SyncState,
        respond_to: oneshot::Sender<Result<(), SyncErr>>,
    },
    SyncIfNotInCooldown {
        respond_to: oneshot::Sender<Result<(), SyncErr>>,
    },
    Sync {
        respond_to: oneshot::Sender<Result<(), SyncErr>>,
    },
    Subscribe {
        respond_to: oneshot::Sender<Result<watch::Receiver<SyncEvent>, SyncErr>>,
    },
}

pub struct Worker<HTTPClientT: ConfigInstancesExt + Send> {
    syncer: SingleThreadSyncer<HTTPClientT>,
    receiver: mpsc::Receiver<WorkerCommand>,
}

impl<HTTPClientT: ConfigInstancesExt + Send> Worker<HTTPClientT> {
    pub fn new(syncer: SingleThreadSyncer<HTTPClientT>, receiver: mpsc::Receiver<WorkerCommand>) -> Self {
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
                WorkerCommand::GetSyncState { respond_to } => {
                    let result = self.syncer.get_sync_state().await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to send state response: {:?}", e);
                    }
                }
                #[cfg(feature = "test")]
                WorkerCommand::SetSyncState { state, respond_to } => {
                    self.syncer.set_sync_state(state);
                    if let Err(e) = respond_to.send(Ok(())) {
                        error!("Actor failed to send set sync state response: {:?}", e);
                    }
                }
                WorkerCommand::SyncIfNotInCooldown { respond_to } => {
                    let result = self.syncer.sync_if_not_in_cooldown().await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to send sync if not in cooldown response: {:?}", e);
                    }
                }
                WorkerCommand::Sync { respond_to } => {
                    let result = self.syncer.sync().await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to send sync response: {:?}", e);
                    }
                }
                WorkerCommand::Subscribe { respond_to } => {
                    let result = self.syncer.subscribe();
                    if respond_to.send(result).is_err() {
                        error!("Actor failed to send subscribe response");
                    }
                }
                
            }
        }
    }
}

#[derive(Debug)]
pub struct Syncer {
    sender: mpsc::Sender<WorkerCommand>,
}

impl Syncer {
    pub fn spawn(
        buffer_size: usize,
        args: SyncerArgs<HTTPClient, TokenManager>,
    ) -> Result<(Self, JoinHandle<()>), SyncErr> {
        let (sender, receiver) = mpsc::channel(buffer_size);
        let worker = Worker {
            syncer: SingleThreadSyncer::new(args),
            receiver,
        };
        let worker_handle = tokio::spawn(worker.run());
        Ok((Self { sender }, worker_handle))
    }

    pub fn new(sender: mpsc::Sender<WorkerCommand>) -> Self {
        Self { sender }
    }
}

impl Syncer {
    #[cfg(feature = "test")]
    pub async fn set_sync_state(&self, state: SyncState) -> Result<(), SyncErr> {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::SetSyncState { state, respond_to: send })
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

impl SyncerExt for Syncer {

    async fn shutdown(&self) -> Result<(), SyncErr> {
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

    async fn get_sync_state(&self) -> Result<SyncState, SyncErr> {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::GetSyncState { respond_to: send })
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

    async fn is_in_cooldown(&self) -> Result<bool, SyncErr> {
        let state = self.get_sync_state().await?;
        Ok(state.is_in_cooldown())
    }

    async fn get_cooldown_ends_at(&self) -> Result<DateTime<Utc>, SyncErr> {
        let state = self.get_sync_state().await?;
        Ok(state.cooldown_ends_at)
    }

    async fn sync_if_not_in_cooldown(&self) -> Result<(), SyncErr> {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::SyncIfNotInCooldown { respond_to: send })
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
        Ok(())
    }

    async fn sync(&self) -> Result<(), SyncErr> {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::Sync {
                respond_to: send,
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

    async fn subscribe(&self) -> Result<watch::Receiver<SyncEvent>, SyncErr> {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::Subscribe { respond_to: send })
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