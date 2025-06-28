// standard library
use std::fmt::Debug;
use std::hash::Hash;
use std::cmp::Eq;

// internal crates
use crate::cache::{
    entry::CacheEntry,
    errors::{CacheErr, ReceiveActorMessageErr, SendActorMessageErr},
    single_thread::SingleThreadCache,
};
use crate::crud::{
    prelude::*,
    errors::{CrudErr, CrudCacheErr},
};
use crate::trace;

// external crates
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::oneshot;
use tracing::{error, info};

// ============================== WORKER COMMANDS ================================== //
type QueryEntryFilter<K, V> = Box<dyn Fn(&CacheEntry<K, V>) -> bool + Send + Sync>;
type QueryValueFilter<V> = Box<dyn Fn(&V) -> bool + Send + Sync>;
type IsDirty<K, V> = Box<dyn Fn(Option<&CacheEntry<K, V>>, &V) -> bool + Send + Sync>;

pub enum WorkerCommand<K, V>
where
    K: Clone + Send + Sync + ToString + Serialize + DeserializeOwned,
    V: Clone + Send + Sync + Serialize + DeserializeOwned,
{
    ReadEntryOptional {
        key: K,
        update_last_accessed: bool,
        respond_to: oneshot::Sender<Result<Option<CacheEntry<K, V>>, CacheErr>>,
    },
    ReadEntry {
        key: K,
        update_last_accessed: bool,
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
        max_size: usize,
        respond_to: oneshot::Sender<Result<(), CacheErr>>,
    },
    Shutdown {
        respond_to: oneshot::Sender<Result<(), CacheErr>>,
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
}

// =================================== WORKER ====================================== //
pub struct Worker<SingleThreadCacheT, K, V>
where
    SingleThreadCacheT: SingleThreadCache<K, V>,
    K: Debug + Clone + ToString + Send + Sync + Serialize + DeserializeOwned + Eq + Hash,
    V: Debug + Clone + Send + Sync + Serialize + DeserializeOwned,
{
    pub cache: SingleThreadCacheT,
    pub receiver: Receiver<WorkerCommand<K, V>>,
}

impl<SingleThreadCacheT, K, V> Worker<SingleThreadCacheT, K, V>
where
    SingleThreadCacheT: SingleThreadCache<K, V>,
    K: Debug + Clone + ToString + Send + Sync + Serialize + DeserializeOwned + Eq + Hash,
    V: Debug + Clone + Send + Sync + Serialize + DeserializeOwned,
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
                WorkerCommand::ReadEntryOptional {
                    key,
                    update_last_accessed,
                    respond_to,
                } => {
                    let result = self
                        .cache
                        .read_entry_optional(&key, update_last_accessed)
                        .await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to read optional cache entry: {:?}", e);
                    }
                }
                WorkerCommand::ReadEntry {
                    key,
                    update_last_accessed,
                    respond_to,
                } => {
                    let result = self.cache.read_entry(&key, update_last_accessed).await;
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
                WorkerCommand::Prune {
                    max_size,
                    respond_to,
                } => {
                    let result = self.cache.prune(max_size).await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to prune cache: {:?}", e);
                    }
                }
                WorkerCommand::FindEntriesWhere {
                    filter,
                    respond_to,
                } => {
                    let result = self.cache.find_entries_where(filter).await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to find all cache entries: {:?}", e);
                    }
                }
                WorkerCommand::FindWhere {
                    filter,
                    respond_to,
                } => {
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
                    let result = self.cache.find_one_entry_optional(filter_name, filter).await;
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
            }
        }
    }
}

// =============================== CONCURRENT CACHE ================================ //
#[derive(Debug)]
pub struct ConcurrentCache<SingleThreadCacheT, K, V>
where
    SingleThreadCacheT: SingleThreadCache<K, V>,
    K: Debug + Clone + Send + Sync + ToString + Serialize + DeserializeOwned + Eq + Hash + 'static,
    V: Debug + Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    sender: Sender<WorkerCommand<K, V>>,
    _phantom: std::marker::PhantomData<SingleThreadCacheT>,
}

impl<SingleThreadCacheT, K, V> ConcurrentCache<SingleThreadCacheT, K, V>
where
    SingleThreadCacheT: SingleThreadCache<K, V>,
    K: Debug + Clone + Send + Sync + ToString + Serialize + DeserializeOwned + Eq + Hash + 'static,
    V: Debug + Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    pub fn new(sender: Sender<WorkerCommand<K, V>>) -> Self {
        Self { sender, _phantom: std::marker::PhantomData }
    }
}

impl<SingleThreadCacheT, K, V> ConcurrentCache<SingleThreadCacheT, K, V>
where
    SingleThreadCacheT: SingleThreadCache<K, V>,
    K: Debug + Clone + Send + Sync + ToString + Serialize + DeserializeOwned + Eq + Hash + 'static,
    V: Debug + Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    pub async fn shutdown(&self) -> Result<(), CacheErr> {
        info!("Shutting down {} cache...", std::any::type_name::<V>());
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::Shutdown { respond_to: send })
            .await
            .map_err(|e| {
                CacheErr::SendActorMessageErr(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                })
            })?;
        recv.await.map_err(|e| {
            CacheErr::ReceiveActorMessageErr(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            })
        })??;
        info!("{} cache shutdown complete", std::any::type_name::<V>());
        Ok(())
    }

    pub async fn read_entry_optional(
        &self,
        key: K,
        update_last_accessed: bool,
    ) -> Result<Option<CacheEntry<K, V>>, CacheErr> {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::ReadEntryOptional {
                key,
                update_last_accessed,
                respond_to: send,
            })
            .await
            .map_err(|e| {
                CacheErr::SendActorMessageErr(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                })
            })?;
        recv.await.map_err(|e| {
            CacheErr::ReceiveActorMessageErr(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            })
        })?
    }

    pub async fn read_entry(
        &self,
        key: K,
        update_last_accessed: bool,
    ) -> Result<CacheEntry<K, V>, CacheErr> {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::ReadEntry {
                key,
                update_last_accessed,
                respond_to: send,
            })
            .await
            .map_err(|e| {
                CacheErr::SendActorMessageErr(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                })
            })?;
        recv.await.map_err(|e| {
            CacheErr::ReceiveActorMessageErr(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            })
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
                CacheErr::SendActorMessageErr(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                })
            })?;
        recv.await.map_err(|e| {
            CacheErr::ReceiveActorMessageErr(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            })
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
                CacheErr::SendActorMessageErr(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                })
            })?;
        recv.await.map_err(|e| {
            CacheErr::ReceiveActorMessageErr(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            })
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
                CacheErr::SendActorMessageErr(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                })
            })?;
        recv.await.map_err(|e| {
            CacheErr::ReceiveActorMessageErr(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            })
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
                CacheErr::SendActorMessageErr(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                })
            })?;
        recv.await.map_err(|e| {
            CacheErr::ReceiveActorMessageErr(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            })
        })?
    }

    pub async fn prune(&self, max_size: usize) -> Result<(), CacheErr> {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::Prune {
                max_size,
                respond_to: send,
            })
            .await
            .map_err(|e| {
                CacheErr::SendActorMessageErr(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                })
            })?;
        recv.await.map_err(|e| {
            CacheErr::ReceiveActorMessageErr(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            })
        })?
    }

    pub async fn find_entries_where<F>(
        &self,
        filter: F,
    ) -> Result<Vec<CacheEntry<K, V>>, CacheErr>
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
                CacheErr::SendActorMessageErr(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                })
            })?;
        recv.await.map_err(|e| {
            CacheErr::ReceiveActorMessageErr(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            })
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
                CacheErr::SendActorMessageErr(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                })
            })?;
        recv.await.map_err(|e| {
            CacheErr::ReceiveActorMessageErr(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            })
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
                CacheErr::SendActorMessageErr(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                })
            })?;
        recv.await.map_err(|e| {
            CacheErr::ReceiveActorMessageErr(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            })
        })?
    }

    async fn find_where_impl<F>(
        &self,
        filter: F,
    ) -> Result<Vec<V>, CacheErr>
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
                CacheErr::SendActorMessageErr(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                })
            })?;
        recv.await.map_err(|e| {
            CacheErr::ReceiveActorMessageErr(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            })
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
                CacheErr::SendActorMessageErr(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                })
            })?;
        recv.await.map_err(|e| {
            CacheErr::ReceiveActorMessageErr(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            })
        })?
    }

    async fn find_one_impl<F>(
        &self,
        filter_name: &'static str,
        filter: F,
    ) -> Result<V, CacheErr>
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
                CacheErr::SendActorMessageErr(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                })
            })?;
        recv.await.map_err(|e| {
            CacheErr::ReceiveActorMessageErr(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            })
        })?
    }
}

// ==================================== FIND ======================================= //
impl<SingleThreadCacheT, K, V> Find<K, V> for ConcurrentCache<SingleThreadCacheT, K, V>
where
    SingleThreadCacheT: SingleThreadCache<K, V>,
    K: Debug + Clone + Send + Sync + ToString + Serialize + DeserializeOwned + Eq + Hash + 'static,
    V: Debug + Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    async fn find_where<F>(&self, filter: F) -> Result<Vec<V>, CrudErr>
    where
        F: Fn(&V) -> bool + Send + Sync + 'static,
    {
        self.find_where_impl(filter).await.map_err(|e| CrudErr::CacheErr(CrudCacheErr {
            source: e,
            trace: trace!(),
        }))
    }

    async fn find_one_optional<F>(
        &self,
        filter_name: &'static str,
        filter: F,
    ) -> Result<Option<V>, CrudErr>
    where
        F: Fn(&V) -> bool + Send + Sync + 'static,
    {
        self.find_one_optional_impl(filter_name, filter).await.map_err(|e| CrudErr::CacheErr(CrudCacheErr {
            source: e,
            trace: trace!(),
        }))
    }

    async fn find_one<F>(
        &self,
        filter_name: &'static str,
        filter: F,
    ) -> Result<V, CrudErr>
    where
        F: Fn(&V) -> bool + Send + Sync + 'static,
    {
        self.find_one_impl(filter_name, filter).await.map_err(|e| CrudErr::CacheErr(CrudCacheErr {
            source: e,
            trace: trace!(),
        }))
    }
}

// ==================================== READ ======================================= //
impl<SingleThreadCacheT, K, V> Read<K, V> for ConcurrentCache<SingleThreadCacheT, K, V>
where
    SingleThreadCacheT: SingleThreadCache<K, V>,
    K: Debug + Clone + Send + Sync + ToString + Serialize + DeserializeOwned + Eq + Hash + 'static,
    V: Debug + Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    async fn read(&self, key: K) -> Result<V, CrudErr> {
        self.read_impl(key).await.map_err(|e| CrudErr::CacheErr(CrudCacheErr {
            source: e,
            trace: trace!(),
        }))
    }

    async fn read_optional(&self, key: K) -> Result<Option<V>, CrudErr> {
        self.read_optional_impl(key).await.map_err(|e| CrudErr::CacheErr(CrudCacheErr {
            source: e,
            trace: trace!(),
        }))
    }
}