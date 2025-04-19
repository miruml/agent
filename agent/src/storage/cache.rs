// std
use std::fmt::Debug;

// internal crates
use crate::filesys::{
    dir::Dir,
    file,
    file::File,
    path::PathExt,
};
use crate::storage::errors::StorageErr;
use crate::trace;

// external crates
use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::oneshot;
use tracing::error;
use futures::future::try_join_all;

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheEntry<K, V> 
where
    K: ToString + Serialize,
    V: Serialize,
{
    pub key: K,
    pub value: V,
    pub last_accessed: DateTime<Utc>,
}

impl<K, V> CacheEntry<K, V>
where
    K: ToString + Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
{
    pub fn new(key: K, value: V) -> Self {
        Self {
            key,
            value,
            last_accessed: Utc::now(),
        }
    }
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
    ) -> Result<Option<CacheEntry<K, V>>, StorageErr> {
        let config_file = self.cache_entry_file(key);
        if !config_file.exists() {
            return Ok(None);
        }

        let entry = config_file.read_json::<CacheEntry<K, V>>().await.map_err(|e| StorageErr::FileSysErr {
            source: e,
            trace: trace!(),
        })?;
        Ok(Some(entry))
    }

    async fn read_entry(
        &self,
        key: &K,
    ) -> Result<CacheEntry<K, V>, StorageErr> {
        let result = self.read_entry_optional(key).await?;
        match result {
            Some(entry) => Ok(entry),
            None => Err(StorageErr::CacheElementNotFound {
                msg: format!("Unable to find cache entry with key: '{}'", key.to_string()),
                trace: trace!(),
            }),
        }
    }

    async fn read_optional(
        &self,
        key: &K,
    ) -> Result<Option<V>, StorageErr> {
        let entry = self.read_entry_optional(key).await?;
        match entry {
            Some(entry) => Ok(Some(entry.value)),
            None => Err(StorageErr::CacheElementNotFound {
                msg: format!("Unable to find cache entry with key: '{}'", key.to_string()),
                trace: trace!(),
            }),
        }
    }

    async fn read(
        &self,
        key: &K,
    ) -> Result<V, StorageErr> {
        Ok(self.read_entry(key).await?.value)
    }

    async fn write(
        &self,
        key: K,
        value: V,
        overwrite: bool,
    ) -> Result<(), StorageErr> {
        let config_file = self.cache_entry_file(&key);
        // important that atomic writes are used here
        let entry = CacheEntry::new(key, value);
        config_file.write_json(
            &entry,
            overwrite,
            // important that atomic writes are used here
            true,
        ).await.map_err(|e| StorageErr::FileSysErr {
            source: e,
            trace: trace!(),
        })?;
        Ok(())
    }

    async fn delete(
        &self,
        key: &K,
    ) -> Result<(), StorageErr> {
        let config_file = self.cache_entry_file(key);
        config_file.delete().await.map_err(|e| StorageErr::FileSysErr {
            source: e,
            trace: trace!(),
        })
    }

    async fn prune(&self, max_size: usize) -> Result<(), StorageErr> {
        // check if there are too many files
        let size = self.size().await?;
        if size <= max_size {
            return Ok(());
        }

        // prune the invalid entries first
        self.prune_invalid_entries().await?;

        // prune by last accessed time
        let entries = self.entries().await?;
        let mut entries = entries.into_iter().collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.last_accessed);
        let num_delete = entries.len() - max_size;
        let futures = entries.into_iter()
            .take(num_delete)
            .map(|entry| async move {
                self.delete(&entry.key).await
            });
        try_join_all(futures).await?;
        Ok(())
    }

    async fn size(&self) -> Result<usize, StorageErr> {
        let files = self.dir.files().await.map_err(|e| StorageErr::FileSysErr {
            source: e,
            trace: trace!(),
        })?;
        Ok(files.len())
    }

    async fn entries(&self) -> Result<Vec<CacheEntry<K, V>>, StorageErr> {
        let files = self.dir.files().await.map_err(|e| StorageErr::FileSysErr {
            source: e,
            trace: trace!(),
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
        let files = self.dir.files().await.map_err(|e| StorageErr::FileSysErr {
            source: e,
            trace: trace!(),
        })?;
        let futures = files.into_iter().map(|file| async move {
            match file.read_json::<CacheEntry<K, V>>().await {
                Ok(_) => Ok(()),
                Err(_) => file.delete().await.map_err(|e| StorageErr::FileSysErr {
                    source: e,
                    trace: trace!(),
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
                WorkerCommand::ReadOptional { key, respond_to } => {
                    let result = self.cache.read_optional(&key).await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to read optional cache entry: {:?}", e);
                    }
                },
                WorkerCommand::Read { key, respond_to } => {
                    let result = self.cache.read(&key).await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to read cache entry: {:?}", e);
                    }
                },
                WorkerCommand::Write { key, value, overwrite, respond_to } => {
                    let result = self.cache.write(key, value, overwrite).await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to write cache entry: {:?}", e);
                    }
                },
                WorkerCommand::Delete { key, respond_to } => {
                    let result = self.cache.delete(&key).await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to delete cache entry: {:?}", e);
                    }
                },
                WorkerCommand::Prune { max_size, respond_to } => {
                    let result = self.cache.prune(max_size).await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to prune cache: {:?}", e);
                    }
                },
            }
        }
    }
}


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
    pub fn spawn(dir: Dir) -> Self {
        let (sender, receiver) = mpsc::channel(64);
        let worker = Worker {
            cache: SingleThreadCache::new(dir),
            receiver,
        };
        tokio::spawn(worker.run());
        Self {
            sender,
        }
    }

    pub async fn read_optional(
        &self,
        key: K,
    ) -> Result<Option<V>, StorageErr> {
        let (send, recv) = oneshot::channel();
        self.sender.send(WorkerCommand::ReadOptional {
            key,
            respond_to: send,
        }).await.map_err(|e| StorageErr::SendActorMessageErr {
            source: Box::new(e),
            trace: trace!(),
        })?;
        recv.await.map_err(|e| StorageErr::ReceiveActorMessageErr {
            source: Box::new(e),
            trace: trace!(),
        })?
    }

    pub async fn read(
        &self,
        key: K,
    ) -> Result<V, StorageErr> {
        let (send, recv) = oneshot::channel();
        self.sender.send(WorkerCommand::Read {
            key,
            respond_to: send,
        }).await.map_err(|e| StorageErr::SendActorMessageErr {
            source: Box::new(e),
            trace: trace!(),
        })?;
        recv.await.map_err(|e| StorageErr::ReceiveActorMessageErr {
            source: Box::new(e),
            trace: trace!(),
        })?
    }

    pub async fn write(
        &self,
        key: K,
        value: V,
        overwrite: bool,
    ) -> Result<(), StorageErr> {
        let (send, recv) = oneshot::channel();
        self.sender.send(WorkerCommand::Write {
            key,
            value,
            overwrite,
            respond_to: send,
        }).await.map_err(|e| StorageErr::SendActorMessageErr {
            source: Box::new(e),
            trace: trace!(),
        })?;
        recv.await.map_err(|e| StorageErr::ReceiveActorMessageErr {
            source: Box::new(e),
            trace: trace!(),
        })?
    }

    pub async fn delete(
        &self,
        key: K,
    ) -> Result<(), StorageErr> {
        let (send, recv) = oneshot::channel();
        self.sender.send(WorkerCommand::Delete {
            key,
            respond_to: send,
        }).await.map_err(|e| StorageErr::SendActorMessageErr {
            source: Box::new(e),
            trace: trace!(),
        })?;
        recv.await.map_err(|e| StorageErr::ReceiveActorMessageErr {
            source: Box::new(e),
            trace: trace!(),
        })?
    }

    pub async fn prune(
        &self,
        max_size: usize,
    ) -> Result<(), StorageErr> {
        let (send, recv) = oneshot::channel();
        self.sender.send(WorkerCommand::Prune {
            max_size,
            respond_to: send,
        }).await.map_err(|e| StorageErr::SendActorMessageErr {
            source: Box::new(e),
            trace: trace!(),
        })?;
        recv.await.map_err(|e| StorageErr::ReceiveActorMessageErr {
            source: Box::new(e),
            trace: trace!(),
        })?
    }
}