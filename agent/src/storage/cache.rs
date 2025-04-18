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
use serde::Serialize;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::oneshot;
use tracing::error;

pub struct CacheEntry<T> 
where
    T: Serialize + DeserializeOwned,
{
    pub value: T,
    pub last_accessed: DateTime<Utc>,
}

impl<T> CacheEntry<T> 
where
    T: Serialize + DeserializeOwned,
{
    pub fn new(value: T) -> Self {
        Self {
            value,
            last_accessed: Utc::now(),
        }
    }
}

// ========================== SINGLE-THREADED IMPLEMENTATION ======================== //
pub struct SingleThreadCache<K, V> 
where
    K: ToString,
    V: Serialize + DeserializeOwned,
{
    dir: Dir,
    _phantom: std::marker::PhantomData<K>,
    _phantom2: std::marker::PhantomData<V>,
}

impl<K, V> SingleThreadCache<K, V> 
where
    K: ToString,
    V: Serialize + DeserializeOwned,
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

    async fn read_optional(
        &self,
        key: &K,
    ) -> Result<Option<V>, StorageErr> {
        let config_file = self.cache_entry_file(key);
        if !config_file.exists() {
            return Ok(None);
        }

        let config = config_file.read_json::<V>().await.map_err(|e| StorageErr::FileSysErr {
            source: e,
            trace: trace!(),
        })?;
        Ok(Some(config))
    }

    async fn read(
        &self,
        key: &K,
    ) -> Result<V, StorageErr> {
        let result = self.read_optional(key).await?;
        match result {
            Some(config) => Ok(config),
            None => Err(StorageErr::CacheElementNotFound {
                msg: format!("Unable to find cache entry with key: '{}'", key.to_string()),
                trace: trace!(),
            }),
        }
    }

    async fn write(
        &self,
        key: &K,
        value: &V,
        overwrite: bool,
    ) -> Result<(), StorageErr> {
        let config_file = self.cache_entry_file(key);
        // important that atomic writes are used here
        config_file.write_json(
            value,
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

    async fn size(&self) -> Result<usize, StorageErr> {
        let files = self.dir.files().await.map_err(|e| StorageErr::FileSysErr {
            source: e,
            trace: trace!(),
        })?;
        Ok(files.len())
    }
}


// ========================== MULTI-THREADED IMPLEMENTATION ======================== //
pub enum WorkerCommand<K, V> 
where
    K: Send + Sync,
    V: Send + Sync,
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
    Size {
        respond_to: oneshot::Sender<Result<usize, StorageErr>>,
    },
}

struct Worker<K, V> 
where
    K: ToString + Send + Sync,
    V: Send + Sync + Serialize + DeserializeOwned,
{
    cache: SingleThreadCache<K, V>,
    receiver: Receiver<WorkerCommand<K, V>>,
}

impl<K, V> Worker<K, V> 
where
    K: ToString + Send + Sync,
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
                    let result = self.cache.write(&key, &value, overwrite).await;
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
                WorkerCommand::Size { respond_to } => {
                    let result = self.cache.size().await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to get cache size: {:?}", e);
                    }
                },
            }
        }
    }
}


pub struct Cache<K, V> 
where
    K: Send + Sync + ToString + 'static,
    V: Debug + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    sender: Sender<WorkerCommand<K, V>>,
}

impl<K, V> Cache<K, V> 
where
    K: Send + Sync + ToString + 'static,
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

    pub async fn size(
        &self,
    ) -> Result<usize, StorageErr> {
        let (send, recv) = oneshot::channel();
        self.sender.send(WorkerCommand::Size {
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