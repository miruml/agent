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
use serde::Deserialize;
use serde::Serialize;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::oneshot;
use tracing::error;

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConcreteConfig {
    #[serde(rename = "concrete_config_id")]
    pub id: String,
    pub created_at: String,
    pub client_id: String,
    pub config_schema_id: String,
    pub concrete_config: serde_json::Value,

    // agent specific fields
    pub config_slug: String,
    pub config_schema_digest: String,
}


// ======================== SINGLE THREADED IMPLEMENTATION ========================= //
struct ConcreteConfigCacheImplementation {
    pub dir: Dir,
}

impl ConcreteConfigCacheImplementation {
    fn new(dir: Dir) -> Self {
        Self { 
            dir, 
        }
    }

    fn config_schema_file(
        &self,
        config_slug: &str,
        config_schema_digest: &str,
    ) -> File {
        let filename = format!(
            "{}_{}.json",
            config_slug,
            config_schema_digest,
        );
        let filename = file::sanitize_filename(&filename);
        self.dir.file(&filename)
    }

    async fn read_optional(
        &self,
        config_slug: &str,
        config_schema_digest: &str,
    ) -> Result<Option<ConcreteConfig>, StorageErr> {
        let config_file = self.config_schema_file(config_slug, config_schema_digest);
        if !config_file.exists() {
            return Ok(None);
        }

        let config = config_file.read_json::<ConcreteConfig>().await.map_err(|e| StorageErr::FileSysErr {
            source: e,
            trace: trace!(),
        })?;
        Ok(Some(config))
    }

    async fn read(
        &self,
        config_slug: &str,
        config_schema_digest: &str,
    ) -> Result<ConcreteConfig, StorageErr> {
        let result = self.read_optional(config_slug, config_schema_digest).await?;
        match result {
            Some(config) => Ok(config),
            None => Err(StorageErr::CacheElementNotFound {
                msg: format!("Unable to find concrete config with slug: '{}' and config schema digest: '{}'", config_slug, config_schema_digest),
                trace: trace!(),
            }),
        }
    }

    async fn write(
        &self,
        concrete_config: &ConcreteConfig,
        overwrite: bool,
    ) -> Result<(), StorageErr> {
        let config_file = self.config_schema_file(
            &concrete_config.config_slug,
            &concrete_config.config_schema_digest,
        );
        // important that atomic writes are used here
        config_file.write_json(
            concrete_config,
            overwrite,
            // important that atomic writes are used here
            true,
        ).await.map_err(|e| StorageErr::FileSysErr {
            source: e,
            trace: trace!(),
        })?;
        Ok(())
    }
}





// ========================== MULTI-THREADED IMPLEMENTATION ======================== //

enum WorkerCommand {
    ReadOptional {
        config_slug: String,
        config_schema_digest: String,
        respond_to: oneshot::Sender<Result<Option<ConcreteConfig>, StorageErr>>,
    },
    Read {
        config_slug: String,
        config_schema_digest: String,
        respond_to: oneshot::Sender<Result<ConcreteConfig, StorageErr>>,
    },
    Write {
        concrete_config: ConcreteConfig,
        overwrite: bool,
        respond_to: oneshot::Sender<Result<(), StorageErr>>,
    },
}

// This struct is responsible for doing the actual work of reading and writing to the
// cache.
struct Worker {
    cache: ConcreteConfigCacheImplementation,
    receiver: Receiver<WorkerCommand>,
}

impl Worker {
    async fn run(mut self) {
        while let Some(cmd) = self.receiver.recv().await {
            match cmd {
                WorkerCommand::ReadOptional { 
                    config_slug,
                    config_schema_digest,
                    respond_to,
                } => {
                    let result = self.cache.read_optional(
                        &config_slug,
                        &config_schema_digest,
                    ).await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to read optional concrete config: {:?}", e);
                    }
                },
                WorkerCommand::Read { 
                    config_slug,
                    config_schema_digest,
                    respond_to,
                } => {
                    let result = self.cache.read(
                        &config_slug,
                        &config_schema_digest,
                    ).await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to read concrete config: {:?}", e);
                    }
                },
                WorkerCommand::Write { 
                    concrete_config,
                    overwrite,
                    respond_to,
                } => {
                    let result = self.cache.write(
                        &concrete_config,
                        overwrite,
                    ).await;
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to write concrete config: {:?}", e);
                    }
                },
            }
        }
    }
}

// This struct is the public interface for the cache. In practice, it just forwards
// commands to the actor and returns the results once they are ready.
#[derive(Clone)]
pub struct ConcreteConfigCache {
    sender: Sender<WorkerCommand>,
}

impl ConcreteConfigCache {
    pub fn spawn(dir: Dir) -> ConcreteConfigCache {
        let (sender, receiver) = mpsc::channel(32);
        let worker = Worker { cache: ConcreteConfigCacheImplementation::new(dir), receiver };

        tokio::spawn(worker.run());
        ConcreteConfigCache { sender }
    }

    pub async fn read_optional(
        &self,
        config_slug: &str,
        config_schema_digest: &str,
    ) -> Result<Option<ConcreteConfig>, StorageErr> {
        let (send, recv) = oneshot::channel();
        self.sender.send(WorkerCommand::ReadOptional {
            config_slug: config_slug.to_string(),
            config_schema_digest: config_schema_digest.to_string(),
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
        config_slug: &str,
        config_schema_digest: &str,
    ) -> Result<ConcreteConfig, StorageErr> {
        let (send, recv) = oneshot::channel();
        self.sender.send(WorkerCommand::Read {
            config_slug: config_slug.to_string(),
            config_schema_digest: config_schema_digest.to_string(),
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
        concrete_config: ConcreteConfig,
        overwrite: bool,
    ) -> Result<(), StorageErr> {
        let (send, recv) = oneshot::channel();
        self.sender.send(WorkerCommand::Write {
            concrete_config,
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
}