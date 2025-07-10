// standard library
use std::collections::HashMap;
use std::fmt::Debug;

// internal crates
use crate::cache::{
    entry::CacheEntry,
    errors::{CacheErr, ReceiveActorMessageErr, SendActorMessageErr},
    single_thread::{CacheKey, CacheValue, SingleThreadCache},
};
use crate::crud::{
    errors::{CrudCacheErr, CrudErr},
    prelude::*,
};
use crate::trace;

// external crates
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::oneshot;
use tracing::{error, info};

pub trait ConcurrentCacheKey: CacheKey + Send + Sync + 'static {}

impl<K> ConcurrentCacheKey for K where K: CacheKey + Send + Sync + 'static {}

pub trait ConcurrentCacheValue: CacheValue + Send + Sync + 'static {}

impl<V> ConcurrentCacheValue for V where V: CacheValue + Send + Sync + 'static {}

// ============================== WORKER COMMANDS ================================== //
type QueryEntryFilter<K, V> = Box<dyn Fn(&CacheEntry<K, V>) -> bool + Send + Sync>;
type QueryValueFilter<V> = Box<dyn Fn(&V) -> bool + Send + Sync>;
type IsDirty<K, V> = Box<dyn Fn(Option<&CacheEntry<K, V>>, &V) -> bool + Send + Sync>;
type CacheEntryMap<K, V> = HashMap<K, CacheEntry<K, V>>;

pub enum WorkerCommand<K, V>
where
    K: Clone + Send + Sync + ToString + Serialize + DeserializeOwned,
    V: Clone + Send + Sync + Serialize + DeserializeOwned,
{
    Shutdown {
        respond_to: oneshot::Sender<Result<(), CacheErr>>,
    },
    ReadEntryOptional {
        key: K,
        respond_to: oneshot::Sender<Result<Option<CacheEntry<K, V>>, CacheErr>>,
    },
    ReadEntry {
        key: K,
        respond_to: oneshot::Sender<Result<CacheEntry<K, V>, CacheErr>>,
    },
    ReadOptional {
        key: K,
        respond_to: oneshot::Sender<Result<Option<V>, CacheErr>>,
    },
    Read {
        key: K,
        respond_to: oneshot::Sender<Result<V, CacheErr>>,
    },
    Write {
        key: K,
        value: V,
        is_dirty: IsDirty<K, V>,
        overwrite: bool,
        respond_to: oneshot::Sender<Result<(), CacheErr>>,
    },
    Delete {
        key: K,
        respond_to: oneshot::Sender<Result<(), CacheErr>>,
    },
    Prune {
        respond_to: oneshot::Sender<Result<(), CacheErr>>,
    },
    Size {
        respond_to: oneshot::Sender<Result<usize, CacheErr>>,
    },
    Entries {
        respond_to: oneshot::Sender<Result<Vec<CacheEntry<K, V>>, CacheErr>>,
    },
    Values {
        respond_to: oneshot::Sender<Result<Vec<V>, CacheErr>>,
    },
    EntryMap {
        respond_to: oneshot::Sender<Result<CacheEntryMap<K, V>, CacheErr>>,
    },
    ValueMap {
        respond_to: oneshot::Sender<Result<HashMap<K, V>, CacheErr>>,
    },
    FindEntriesWhere {
        filter: QueryEntryFilter<K, V>,
        respond_to: oneshot::Sender<Result<Vec<CacheEntry<K, V>>, CacheErr>>,
    },
    FindWhere {
        filter: QueryValueFilter<V>,
        respond_to: oneshot::Sender<Result<Vec<V>, CacheErr>>,
    },
    FindOneEntryOptional {
        filter_name: &'static str,
        filter: QueryEntryFilter<K, V>,
        respond_to: oneshot::Sender<Result<Option<CacheEntry<K, V>>, CacheErr>>,
    },
    FindOneOptional {
        filter_name: &'static str,
        filter: QueryValueFilter<V>,
        respond_to: oneshot::Sender<Result<Option<V>, CacheErr>>,
    },
    FindOneEntry {
        filter_name: &'static str,
        filter: QueryEntryFilter<K, V>,
        respond_to: oneshot::Sender<Result<CacheEntry<K, V>, CacheErr>>,
    },
    FindOne {
        filter_name: &'static str,
        filter: QueryValueFilter<V>,
        respond_to: oneshot::Sender<Result<V, CacheErr>>,
    },
    GetDirtyEntries {
        respond_to: oneshot::Sender<Result<Vec<CacheEntry<K, V>>, CacheErr>>,
    },
}

// =================================== WORKER ====================================== //
pub struct Worker<SingleThreadCacheT, K, V>
where
    SingleThreadCacheT: SingleThreadCache<K, V>,
    K: ConcurrentCacheKey,
    V: ConcurrentCacheValue,
{
    pub cache: SingleThreadCacheT,
    pub receiver: Receiver<WorkerCommand<K, V>>,
}

impl<SingleThreadCacheT, K, V> Worker<SingleThreadCacheT, K, V>
where
    SingleThreadCacheT: SingleThreadCache<K, V>,
    K: ConcurrentCacheKey,
    V: ConcurrentCacheValue,
{
    pub async fn run(mut self) {
        while let Some(cmd) = self.receiver.recv().await {
            match cmd {
                WorkerCommand::Shutdown { respond_to } => {
                    if let Err(e) = respond_to.send(Ok(())) {
                        error!("Actor failed to send shutdown response: {:?}", e);
                    }
                    break;
                }
                WorkerCommand::ReadEntryOptional { key, respond_to } => {
                    let result = self.cache.read_entry_optional(&key).await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to read optional cache entry: {:?}", e);
                    }
                }
                WorkerCommand::ReadEntry { key, respond_to } => {
                    let result = self.cache.read_entry(&key).await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to read cache entry: {:?}", e);
                    }
                }
                WorkerCommand::ReadOptional { key, respond_to } => {
                    let result = self.cache.read_optional(&key).await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to read optional cache entry: {:?}", e);
                    }
                }
                WorkerCommand::Read { key, respond_to } => {
                    let result = self.cache.read(&key).await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to read cache entry: {:?}", e);
                    }
                }
                WorkerCommand::Write {
                    key,
                    value,
                    is_dirty,
                    overwrite,
                    respond_to,
                } => {
                    let result = self.cache.write(key, value, is_dirty, overwrite).await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to write cache entry: {:?}", e);
                    }
                }
                WorkerCommand::Delete { key, respond_to } => {
                    let result = self.cache.delete(&key).await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to delete cache entry: {:?}", e);
                    }
                }
                WorkerCommand::Prune { respond_to } => {
                    let result = self.cache.prune().await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to prune cache: {:?}", e);
                    }
                }
                WorkerCommand::Size { respond_to } => {
                    let result = self.cache.size().await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to get cache size: {:?}", e);
                    }
                }
                WorkerCommand::Entries { respond_to } => {
                    let result = self.cache.entries().await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to get cache entries: {:?}", e);
                    }
                }
                WorkerCommand::Values { respond_to } => {
                    let result = self.cache.values().await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to get cache values: {:?}", e);
                    }
                }
                WorkerCommand::EntryMap { respond_to } => {
                    let result = self.cache.entry_map().await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to get cache entry map: {:?}", e);
                    }
                }
                WorkerCommand::ValueMap { respond_to } => {
                    let result = self.cache.value_map().await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to get cache value map: {:?}", e);
                    }
                }
                WorkerCommand::FindEntriesWhere { filter, respond_to } => {
                    let result = self.cache.find_entries_where(filter).await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to find all cache entries: {:?}", e);
                    }
                }
                WorkerCommand::FindWhere { filter, respond_to } => {
                    let result = self.cache.find_where(filter).await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to find all cache entries: {:?}", e);
                    }
                }
                WorkerCommand::FindOneEntryOptional {
                    filter_name,
                    filter,
                    respond_to,
                } => {
                    let result = self
                        .cache
                        .find_one_entry_optional(filter_name, filter)
                        .await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to find one cache entry: {:?}", e);
                    }
                }
                WorkerCommand::FindOneOptional {
                    filter_name,
                    filter,
                    respond_to,
                } => {
                    let result = self.cache.find_one_optional(filter_name, filter).await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to find one cache entry: {:?}", e);
                    }
                }
                WorkerCommand::FindOneEntry {
                    filter_name,
                    filter,
                    respond_to,
                } => {
                    let result = self.cache.find_one_entry(filter_name, filter).await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to find one cache entry: {:?}", e);
                    }
                }
                WorkerCommand::FindOne {
                    filter_name,
                    filter,
                    respond_to,
                } => {
                    let result = self.cache.find_one(filter_name, filter).await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to find one cache entry: {:?}", e);
                    }
                }
                WorkerCommand::GetDirtyEntries { respond_to } => {
                    let result = self.cache.get_dirty_entries().await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to get dirty entries: {:?}", e);
                    }
                }
            }
        }
    }
}

// =============================== CONCURRENT CACHE ================================ //
#[derive(Debug)]
pub struct ConcurrentCache<SingleThreadCacheT, K, V>
where
    SingleThreadCacheT: SingleThreadCache<K, V>,
    K: ConcurrentCacheKey,
    V: ConcurrentCacheValue,
{
    sender: Sender<WorkerCommand<K, V>>,
    _phantom: std::marker::PhantomData<SingleThreadCacheT>,
}

impl<SingleThreadCacheT, K, V> ConcurrentCache<SingleThreadCacheT, K, V>
where
    SingleThreadCacheT: SingleThreadCache<K, V>,
    K: ConcurrentCacheKey,
    V: ConcurrentCacheValue,
{
    pub fn new(sender: Sender<WorkerCommand<K, V>>) -> Self {
        Self {
            sender,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<SingleThreadCacheT, K, V> ConcurrentCache<SingleThreadCacheT, K, V>
where
    SingleThreadCacheT: SingleThreadCache<K, V>,
    K: ConcurrentCacheKey,
    V: ConcurrentCacheValue,
{
    pub async fn shutdown(&self) -> Result<(), CacheErr> {
        info!("Shutting down {} cache...", std::any::type_name::<V>());
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::Shutdown { respond_to: send })
            .await
            .map_err(|e| {
                CacheErr::SendActorMessageErr(Box::new(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                }))
            })?;
        recv.await.map_err(|e| {
            CacheErr::ReceiveActorMessageErr(Box::new(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            }))
        })??;
        info!("{} cache shutdown complete", std::any::type_name::<V>());
        Ok(())
    }

    pub async fn read_entry_optional(&self, key: K) -> Result<Option<CacheEntry<K, V>>, CacheErr> {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::ReadEntryOptional {
                key,
                respond_to: send,
            })
            .await
            .map_err(|e| {
                CacheErr::SendActorMessageErr(Box::new(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                }))
            })?;
        recv.await.map_err(|e| {
            CacheErr::ReceiveActorMessageErr(Box::new(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            }))
        })?
    }

    pub async fn read_entry(&self, key: K) -> Result<CacheEntry<K, V>, CacheErr> {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::ReadEntry {
                key,
                respond_to: send,
            })
            .await
            .map_err(|e| {
                CacheErr::SendActorMessageErr(Box::new(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                }))
            })?;
        recv.await.map_err(|e| {
            CacheErr::ReceiveActorMessageErr(Box::new(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            }))
        })?
    }

    async fn read_optional_impl(&self, key: K) -> Result<Option<V>, CacheErr> {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::ReadOptional {
                key,
                respond_to: send,
            })
            .await
            .map_err(|e| {
                CacheErr::SendActorMessageErr(Box::new(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                }))
            })?;
        recv.await.map_err(|e| {
            CacheErr::ReceiveActorMessageErr(Box::new(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            }))
        })?
    }

    async fn read_impl(&self, key: K) -> Result<V, CacheErr> {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::Read {
                key,
                respond_to: send,
            })
            .await
            .map_err(|e| {
                CacheErr::SendActorMessageErr(Box::new(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                }))
            })?;
        recv.await.map_err(|e| {
            CacheErr::ReceiveActorMessageErr(Box::new(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            }))
        })?
    }

    pub async fn write<F>(
        &self,
        key: K,
        value: V,
        is_dirty: F,
        overwrite: bool,
    ) -> Result<(), CacheErr>
    where
        F: Fn(Option<&CacheEntry<K, V>>, &V) -> bool + Send + Sync + 'static,
    {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::Write {
                key,
                value,
                is_dirty: Box::new(is_dirty),
                overwrite,
                respond_to: send,
            })
            .await
            .map_err(|e| {
                CacheErr::SendActorMessageErr(Box::new(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                }))
            })?;
        recv.await.map_err(|e| {
            CacheErr::ReceiveActorMessageErr(Box::new(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            }))
        })?
    }

    pub async fn delete(&self, key: K) -> Result<(), CacheErr> {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::Delete {
                key,
                respond_to: send,
            })
            .await
            .map_err(|e| {
                CacheErr::SendActorMessageErr(Box::new(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                }))
            })?;
        recv.await.map_err(|e| {
            CacheErr::ReceiveActorMessageErr(Box::new(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            }))
        })?
    }

    pub async fn prune(&self) -> Result<(), CacheErr> {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::Prune { respond_to: send })
            .await
            .map_err(|e| {
                CacheErr::SendActorMessageErr(Box::new(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                }))
            })?;
        recv.await.map_err(|e| {
            CacheErr::ReceiveActorMessageErr(Box::new(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            }))
        })?
    }

    pub async fn size(&self) -> Result<usize, CacheErr> {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::Size { respond_to: send })
            .await
            .map_err(|e| {
                CacheErr::SendActorMessageErr(Box::new(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                }))
            })?;
        recv.await.map_err(|e| {
            CacheErr::ReceiveActorMessageErr(Box::new(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            }))
        })?
    }

    pub async fn entries(&self) -> Result<Vec<CacheEntry<K, V>>, CacheErr> {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::Entries { respond_to: send })
            .await
            .map_err(|e| {
                CacheErr::SendActorMessageErr(Box::new(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                }))
            })?;
        recv.await.map_err(|e| {
            CacheErr::ReceiveActorMessageErr(Box::new(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            }))
        })?
    }

    pub async fn values(&self) -> Result<Vec<V>, CacheErr> {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::Values { respond_to: send })
            .await
            .map_err(|e| {
                CacheErr::SendActorMessageErr(Box::new(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                }))
            })?;
        recv.await.map_err(|e| {
            CacheErr::ReceiveActorMessageErr(Box::new(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            }))
        })?
    }

    pub async fn entry_map(&self) -> Result<HashMap<K, CacheEntry<K, V>>, CacheErr> {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::EntryMap { respond_to: send })
            .await
            .map_err(|e| {
                CacheErr::SendActorMessageErr(Box::new(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                }))
            })?;
        recv.await.map_err(|e| {
            CacheErr::ReceiveActorMessageErr(Box::new(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            }))
        })?
    }

    pub async fn value_map(&self) -> Result<HashMap<K, V>, CacheErr> {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::ValueMap { respond_to: send })
            .await
            .map_err(|e| {
                CacheErr::SendActorMessageErr(Box::new(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                }))
            })?;
        recv.await.map_err(|e| {
            CacheErr::ReceiveActorMessageErr(Box::new(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            }))
        })?
    }

    pub async fn find_entries_where<F>(&self, filter: F) -> Result<Vec<CacheEntry<K, V>>, CacheErr>
    where
        F: Fn(&CacheEntry<K, V>) -> bool + Send + Sync + 'static,
    {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::FindEntriesWhere {
                filter: Box::new(filter),
                respond_to: send,
            })
            .await
            .map_err(|e| {
                CacheErr::SendActorMessageErr(Box::new(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                }))
            })?;
        recv.await.map_err(|e| {
            CacheErr::ReceiveActorMessageErr(Box::new(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            }))
        })?
    }

    pub async fn find_one_entry_optional<F>(
        &self,
        filter_name: &'static str,
        filter: F,
    ) -> Result<Option<CacheEntry<K, V>>, CacheErr>
    where
        F: Fn(&CacheEntry<K, V>) -> bool + Send + Sync + 'static,
    {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::FindOneEntryOptional {
                filter_name,
                filter: Box::new(filter),
                respond_to: send,
            })
            .await
            .map_err(|e| {
                CacheErr::SendActorMessageErr(Box::new(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                }))
            })?;
        recv.await.map_err(|e| {
            CacheErr::ReceiveActorMessageErr(Box::new(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            }))
        })?
    }

    pub async fn find_one_entry<F>(
        &self,
        filter_name: &'static str,
        filter: F,
    ) -> Result<CacheEntry<K, V>, CacheErr>
    where
        F: Fn(&CacheEntry<K, V>) -> bool + Send + Sync + 'static,
    {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::FindOneEntry {
                filter_name,
                filter: Box::new(filter),
                respond_to: send,
            })
            .await
            .map_err(|e| {
                CacheErr::SendActorMessageErr(Box::new(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                }))
            })?;
        recv.await.map_err(|e| {
            CacheErr::ReceiveActorMessageErr(Box::new(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            }))
        })?
    }

    async fn find_where_impl<F>(&self, filter: F) -> Result<Vec<V>, CacheErr>
    where
        F: Fn(&V) -> bool + Send + Sync + 'static,
    {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::FindWhere {
                filter: Box::new(filter),
                respond_to: send,
            })
            .await
            .map_err(|e| {
                CacheErr::SendActorMessageErr(Box::new(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                }))
            })?;
        recv.await.map_err(|e| {
            CacheErr::ReceiveActorMessageErr(Box::new(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            }))
        })?
    }

    async fn find_one_optional_impl<F>(
        &self,
        filter_name: &'static str,
        filter: F,
    ) -> Result<Option<V>, CacheErr>
    where
        F: Fn(&V) -> bool + Send + Sync + 'static,
    {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::FindOneOptional {
                filter_name,
                filter: Box::new(filter),
                respond_to: send,
            })
            .await
            .map_err(|e| {
                CacheErr::SendActorMessageErr(Box::new(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                }))
            })?;
        recv.await.map_err(|e| {
            CacheErr::ReceiveActorMessageErr(Box::new(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            }))
        })?
    }

    async fn find_one_impl<F>(&self, filter_name: &'static str, filter: F) -> Result<V, CacheErr>
    where
        F: Fn(&V) -> bool + Send + Sync + 'static,
    {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::FindOne {
                filter_name,
                filter: Box::new(filter),
                respond_to: send,
            })
            .await
            .map_err(|e| {
                CacheErr::SendActorMessageErr(Box::new(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                }))
            })?;
        recv.await.map_err(|e| {
            CacheErr::ReceiveActorMessageErr(Box::new(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            }))
        })?
    }

    pub async fn get_dirty_entries(&self) -> Result<Vec<CacheEntry<K, V>>, CacheErr> {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::GetDirtyEntries { respond_to: send })
            .await
            .map_err(|e| {
                CacheErr::SendActorMessageErr(Box::new(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                }))
            })?;
        recv.await.map_err(|e| {
            CacheErr::ReceiveActorMessageErr(Box::new(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            }))
        })?
    }
}

// ==================================== FIND ======================================= //
impl<SingleThreadCacheT, K, V> Find<K, V> for ConcurrentCache<SingleThreadCacheT, K, V>
where
    SingleThreadCacheT: SingleThreadCache<K, V>,
    K: ConcurrentCacheKey,
    V: ConcurrentCacheValue,
{
    async fn find_where<F>(&self, filter: F) -> Result<Vec<V>, CrudErr>
    where
        F: Fn(&V) -> bool + Send + Sync + 'static,
    {
        self.find_where_impl(filter).await.map_err(|e| {
            CrudErr::CacheErr(Box::new(CrudCacheErr {
                source: e,
                trace: trace!(),
            }))
        })
    }

    async fn find_one_optional<F>(
        &self,
        filter_name: &'static str,
        filter: F,
    ) -> Result<Option<V>, CrudErr>
    where
        F: Fn(&V) -> bool + Send + Sync + 'static,
    {
        self.find_one_optional_impl(filter_name, filter)
            .await
            .map_err(|e| {
                CrudErr::CacheErr(Box::new(CrudCacheErr {
                    source: e,
                    trace: trace!(),
                }))
            })
    }

    async fn find_one<F>(&self, filter_name: &'static str, filter: F) -> Result<V, CrudErr>
    where
        F: Fn(&V) -> bool + Send + Sync + 'static,
    {
        self.find_one_impl(filter_name, filter).await.map_err(|e| {
            CrudErr::CacheErr(Box::new(CrudCacheErr {
                source: e,
                trace: trace!(),
            }))
        })
    }
}

// ==================================== READ ======================================= //
impl<SingleThreadCacheT, K, V> Read<K, V> for ConcurrentCache<SingleThreadCacheT, K, V>
where
    SingleThreadCacheT: SingleThreadCache<K, V>,
    K: ConcurrentCacheKey,
    V: ConcurrentCacheValue,
{
    async fn read(&self, key: K) -> Result<V, CrudErr> {
        self.read_impl(key).await.map_err(|e| {
            CrudErr::CacheErr(Box::new(CrudCacheErr {
                source: e,
                trace: trace!(),
            }))
        })
    }

    async fn read_optional(&self, key: K) -> Result<Option<V>, CrudErr> {
        self.read_optional_impl(key).await.map_err(|e| {
            CrudErr::CacheErr(Box::new(CrudCacheErr {
                source: e,
                trace: trace!(),
            }))
        })
    }
}
