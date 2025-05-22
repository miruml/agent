// standard library
use std::fmt::Debug;

// internal crates
use crate::filesys::{dir::Dir, file, file::File, path::PathExt};
use crate::storage::errors::{
    CacheElementNotFound, ReceiveActorMessageErr, SendActorMessageErr, StorageErr,
    StorageFileSysErr,
};
use crate::trace;

// external crates
use chrono::{DateTime, Utc};
use futures::future::try_join_all;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tracing::{error, info};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CacheEntry<K, V>
where
    K: ToString + Serialize,
    V: Serialize,
{
    pub key: K,
    pub value: V,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
}

// ========================== SINGLE-THREADED IMPLEMENTATION ======================== //
pub struct SingleThreadCache<K, V>
where
    K: Debug + ToString,
    V: Debug + Serialize + DeserializeOwned,
{
    dir: Dir,
    _phantom: std::marker::PhantomData<K>,
    _phantom2: std::marker::PhantomData<V>,
}

impl<K, V> SingleThreadCache<K, V>
where
    K: Debug + ToString + Serialize + DeserializeOwned,
    V: Debug + Serialize + DeserializeOwned,
{
    pub fn new(dir: Dir) -> Self {
        Self {
            dir,
            _phantom: std::marker::PhantomData,
            _phantom2: std::marker::PhantomData,
        }
    }

    fn cache_entry_file(&self, key: &K) -> File {
        let mut filename = format!("{}.json", key.to_string());
        filename = file::sanitize_filename(&filename);
        self.dir.file(&filename)
    }

    async fn read_entry_optional(
        &self,
        key: &K,
        update_last_accessed: bool,
    ) -> Result<Option<CacheEntry<K, V>>, StorageErr> {
        let entry_file = self.cache_entry_file(key);
        if !entry_file.exists() {
            return Ok(None);
        }

        // read the entry
        let mut entry = entry_file
            .read_json::<CacheEntry<K, V>>()
            .await
            .map_err(|e| {
                StorageErr::FileSysErr(StorageFileSysErr {
                    source: e,
                    trace: trace!(),
                })
            })?;

        // update the last accessed time
        if update_last_accessed {
            self.update_last_accessed(&mut entry).await?;
        }

        Ok(Some(entry))
    }

    async fn read_entry(
        &self,
        key: &K,
        update_last_accessed: bool,
    ) -> Result<CacheEntry<K, V>, StorageErr> {
        let result = self.read_entry_optional(key, update_last_accessed).await?;
        match result {
            Some(entry) => Ok(entry),
            None => Err(StorageErr::CacheElementNotFound(CacheElementNotFound {
                msg: format!("Unable to find cache entry with key: '{}'", key.to_string()),
                trace: trace!(),
            })),
        }
    }

    async fn read_optional(&self, key: &K) -> Result<Option<V>, StorageErr> {
        let entry = self.read_entry_optional(key, true).await?;
        match entry {
            Some(entry) => Ok(Some(entry.value)),
            None => Ok(None),
        }
    }

    async fn read(&self, key: &K) -> Result<V, StorageErr> {
        Ok(self.read_entry(key, true).await?.value)
    }

    async fn write_entry(
        &self,
        entry: &CacheEntry<K, V>,
        overwrite: bool,
    ) -> Result<(), StorageErr> {
        let entry_file = self.cache_entry_file(&entry.key);
        entry_file
            .write_json(
                &entry, overwrite, true, // important that atomic writes are used here
            )
            .await
            .map_err(|e| {
                StorageErr::FileSysErr(StorageFileSysErr {
                    source: e,
                    trace: trace!(),
                })
            })?;
        Ok(())
    }

    async fn write(&self, key: K, value: V, overwrite: bool) -> Result<(), StorageErr> {
        // if the entry already exists, keep the original created_at time
        let (created_at, last_accessed) = match self.read_entry_optional(&key, false).await? {
            Some(existing_entry) => (existing_entry.created_at, Utc::now()),
            None => {
                let now = Utc::now();
                (now, now)
            }
        };
        let entry = CacheEntry {
            key,
            value,
            created_at,
            last_accessed,
        };

        // write the entry
        self.write_entry(&entry, overwrite).await?;
        Ok(())
    }

    async fn update_last_accessed(&self, entry: &mut CacheEntry<K, V>) -> Result<(), StorageErr> {
        entry.last_accessed = Utc::now();
        self.write_entry(entry, true).await?;
        Ok(())
    }

    async fn delete(&self, key: &K) -> Result<(), StorageErr> {
        let entry_file = self.cache_entry_file(key);
        entry_file.delete().await.map_err(|e| {
            StorageErr::FileSysErr(StorageFileSysErr {
                source: e,
                trace: trace!(),
            })
        })?;
        Ok(())
    }

    async fn prune(&self, max_size: usize) -> Result<(), StorageErr> {
        // check if there are too many files
        let size = self.size().await?;
        if size <= max_size {
            return Ok(());
        }

        info!(
            "Pruning cache {} from {:?} entries to {:?} entries...",
            std::any::type_name::<V>(),
            size,
            max_size
        );

        // prune the invalid entries first
        self.prune_invalid_entries().await?;

        // prune by last accessed time
        let entries = self.entries().await?;
        let mut entries = entries.into_iter().collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.last_accessed);
        let num_delete = entries.len() - max_size;
        let futures = entries
            .into_iter()
            .take(num_delete)
            .map(|entry| async move { self.delete(&entry.key).await });
        try_join_all(futures).await?;
        Ok(())
    }

    async fn size(&self) -> Result<usize, StorageErr> {
        if !self.dir.exists() {
            return Ok(0);
        }
        let files = self.dir.files().await.map_err(|e| {
            StorageErr::FileSysErr(StorageFileSysErr {
                source: e,
                trace: trace!(),
            })
        })?;
        Ok(files.len())
    }

    async fn entries(&self) -> Result<Vec<CacheEntry<K, V>>, StorageErr> {
        let files = self.dir.files().await.map_err(|e| {
            StorageErr::FileSysErr(StorageFileSysErr {
                source: e,
                trace: trace!(),
            })
        })?;
        let futures = files.into_iter().map(|file| async move {
            match file.read_json::<CacheEntry<K, V>>().await {
                Ok(entry) => Ok(Some(entry)),
                Err(_) => Ok(None),
            }
        });
        let entries = try_join_all(futures).await?;
        Ok(entries.into_iter().flatten().collect())
    }

    async fn prune_invalid_entries(&self) -> Result<(), StorageErr> {
        let files = self.dir.files().await.map_err(|e| {
            StorageErr::FileSysErr(StorageFileSysErr {
                source: e,
                trace: trace!(),
            })
        })?;
        let futures = files.into_iter().map(|file| async move {
            match file.read_json::<CacheEntry<K, V>>().await {
                Ok(_) => Ok(()),
                Err(_) => file.delete().await.map_err(|e| {
                    StorageErr::FileSysErr(StorageFileSysErr {
                        source: e,
                        trace: trace!(),
                    })
                }),
            }
        });
        try_join_all(futures).await?;
        Ok(())
    }
}

// ========================== MULTI-THREADED IMPLEMENTATION ======================== //
enum WorkerCommand<K, V>
where
    K: Send + Sync + ToString + Serialize + DeserializeOwned,
    V: Send + Sync + Serialize + DeserializeOwned,
{
    ReadEntryOptional {
        key: K,
        update_last_accessed: bool,
        respond_to: oneshot::Sender<Result<Option<CacheEntry<K, V>>, StorageErr>>,
    },
    ReadEntry {
        key: K,
        update_last_accessed: bool,
        respond_to: oneshot::Sender<Result<CacheEntry<K, V>, StorageErr>>,
    },
    ReadOptional {
        key: K,
        respond_to: oneshot::Sender<Result<Option<V>, StorageErr>>,
    },
    Read {
        key: K,
        respond_to: oneshot::Sender<Result<V, StorageErr>>,
    },
    Write {
        key: K,
        value: V,
        overwrite: bool,
        respond_to: oneshot::Sender<Result<(), StorageErr>>,
    },
    Delete {
        key: K,
        respond_to: oneshot::Sender<Result<(), StorageErr>>,
    },
    Prune {
        max_size: usize,
        respond_to: oneshot::Sender<Result<(), StorageErr>>,
    },
    Shutdown {
        respond_to: oneshot::Sender<Result<(), StorageErr>>,
    },
}

struct Worker<K, V>
where
    K: Debug + ToString + Send + Sync + Serialize + DeserializeOwned,
    V: Debug + Send + Sync + Serialize + DeserializeOwned,
{
    cache: SingleThreadCache<K, V>,
    receiver: Receiver<WorkerCommand<K, V>>,
}

impl<K, V> Worker<K, V>
where
    K: Debug + ToString + Send + Sync + Serialize + DeserializeOwned,
    V: Debug + Send + Sync + Serialize + DeserializeOwned,
{
    async fn run(mut self) {
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
                    overwrite,
                    respond_to,
                } => {
                    let result = self.cache.write(key, value, overwrite).await;
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
            }
        }
    }
}

#[derive(Debug)]
pub struct Cache<K, V>
where
    K: Send + Sync + ToString + Serialize + DeserializeOwned + 'static,
    V: Debug + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    sender: Sender<WorkerCommand<K, V>>,
}

impl<K, V> Cache<K, V>
where
    K: Debug + Send + Sync + ToString + Serialize + DeserializeOwned + 'static,
    V: Debug + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    pub fn spawn(dir: Dir) -> (Self, JoinHandle<()>) {
        let (sender, receiver) = mpsc::channel(64);
        let worker = Worker {
            cache: SingleThreadCache::new(dir),
            receiver,
        };
        let worker_handle = tokio::spawn(worker.run());
        (Self { sender }, worker_handle)
    }

    pub async fn shutdown(&self) -> Result<(), StorageErr> {
        info!("Shutting down {} cache...", std::any::type_name::<V>());
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::Shutdown { respond_to: send })
            .await
            .map_err(|e| {
                StorageErr::SendActorMessageErr(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                })
            })?;
        recv.await.map_err(|e| {
            StorageErr::ReceiveActorMessageErr(ReceiveActorMessageErr {
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
    ) -> Result<Option<CacheEntry<K, V>>, StorageErr> {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::ReadEntryOptional {
                key,
                update_last_accessed,
                respond_to: send,
            })
            .await
            .map_err(|e| {
                StorageErr::SendActorMessageErr(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                })
            })?;
        recv.await.map_err(|e| {
            StorageErr::ReceiveActorMessageErr(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            })
        })?
    }

    pub async fn read_entry(
        &self,
        key: K,
        update_last_accessed: bool,
    ) -> Result<CacheEntry<K, V>, StorageErr> {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::ReadEntry {
                key,
                update_last_accessed,
                respond_to: send,
            })
            .await
            .map_err(|e| {
                StorageErr::SendActorMessageErr(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                })
            })?;
        recv.await.map_err(|e| {
            StorageErr::ReceiveActorMessageErr(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            })
        })?
    }

    pub async fn read_optional(&self, key: K) -> Result<Option<V>, StorageErr> {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::ReadOptional {
                key,
                respond_to: send,
            })
            .await
            .map_err(|e| {
                StorageErr::SendActorMessageErr(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                })
            })?;
        recv.await.map_err(|e| {
            StorageErr::ReceiveActorMessageErr(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            })
        })?
    }

    pub async fn read(&self, key: K) -> Result<V, StorageErr> {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::Read {
                key,
                respond_to: send,
            })
            .await
            .map_err(|e| {
                StorageErr::SendActorMessageErr(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                })
            })?;
        recv.await.map_err(|e| {
            StorageErr::ReceiveActorMessageErr(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            })
        })?
    }

    pub async fn write(&self, key: K, value: V, overwrite: bool) -> Result<(), StorageErr> {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::Write {
                key,
                value,
                overwrite,
                respond_to: send,
            })
            .await
            .map_err(|e| {
                StorageErr::SendActorMessageErr(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                })
            })?;
        recv.await.map_err(|e| {
            StorageErr::ReceiveActorMessageErr(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            })
        })?
    }

    pub async fn delete(&self, key: K) -> Result<(), StorageErr> {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::Delete {
                key,
                respond_to: send,
            })
            .await
            .map_err(|e| {
                StorageErr::SendActorMessageErr(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                })
            })?;
        recv.await.map_err(|e| {
            StorageErr::ReceiveActorMessageErr(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            })
        })?
    }

    pub async fn prune(&self, max_size: usize) -> Result<(), StorageErr> {
        let (send, recv) = oneshot::channel();
        self.sender
            .send(WorkerCommand::Prune {
                max_size,
                respond_to: send,
            })
            .await
            .map_err(|e| {
                StorageErr::SendActorMessageErr(SendActorMessageErr {
                    source: Box::new(e),
                    trace: trace!(),
                })
            })?;
        recv.await.map_err(|e| {
            StorageErr::ReceiveActorMessageErr(ReceiveActorMessageErr {
                source: Box::new(e),
                trace: trace!(),
            })
        })?
    }
}
