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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ConfigSchemaDigests {
    pub raw: String,
    pub resolved: String,
}


// ============================== SYNC IMPLEMENTATION ============================= //

pub struct SyncConfigSchemaDigestCache {
    pub dir: Dir,
}

impl SyncConfigSchemaDigestCache {
    pub fn new(dir: Dir) -> Self {
        Self { dir }
    }

    fn digest_file(&self, raw_digest: &str) -> File {
        let mut filename = format!("{}.json", raw_digest);
        filename = file::sanitize_filename(&filename);
        self.dir.file(&filename)
    }

    pub fn read_optional(
        &self,
        raw_digest: &str,
    ) -> Result<Option<ConfigSchemaDigests>, StorageErr> {
        let digest_file = self.digest_file(raw_digest);
        if !digest_file.exists() {
            return Ok(None);
        }

        let digests = digest_file.read_json::<ConfigSchemaDigests>().map_err(|e| StorageErr::FileSysErr {
            source: e,
            trace: trace!(),
        })?;
        Ok(Some(digests))
    }

    pub fn read(
        &self,
        raw_digest: &str,
    ) -> Result<ConfigSchemaDigests, StorageErr> {
        let result = self.read_optional(raw_digest)?;
        match result {
            Some(digests) => Ok(digests),
            None => Err(StorageErr::CacheElementNotFound {
                msg: format!("Unable to find config schema digest cache data with raw digest: '{}'", raw_digest),
                trace: trace!(),
            }),
        }
    }

    pub fn write(
        &self,
        digests: ConfigSchemaDigests,
        overwrite: bool,
    ) -> Result<(), StorageErr> {
        let digest_file = self.digest_file(digests.raw.as_str());
        digest_file.write_json(
            &digests,
            overwrite,
        ).map_err(|e| StorageErr::FileSysErr {
            source: e,
            trace: trace!(),
        })
    }


}






// ============================== ASYNC IMPLEMENTATION ============================= //

// Commands that can be sent to the actor
enum WorkerCommand {
    Read {
        raw_digest: String,
        respond_to: oneshot::Sender<Result<ConfigSchemaDigests, StorageErr>>,
    },
    ReadOptional {
        raw_digest: String,
        respond_to: oneshot::Sender<Result<Option<ConfigSchemaDigests>, StorageErr>>,
    },
    Write {
        digests: ConfigSchemaDigests,
        overwrite: bool,
        respond_to: oneshot::Sender<Result<(), StorageErr>>,
    },
}

// This struct is responsible for doing the actual work of reading and writing to the
// cache.
struct Worker {
    cache: SyncConfigSchemaDigestCache,
    receiver: Receiver<WorkerCommand>,
}

impl Worker {
    async fn run(mut self) {
        while let Some(cmd) = self.receiver.recv().await {
            match cmd {
                WorkerCommand::ReadOptional {
                    raw_digest,
                    respond_to,
                } => {
                    let result = self.cache.read_optional(&raw_digest);
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to read config schema digests: {:?}", e);
                    }
                },
                WorkerCommand::Read { 
                    raw_digest,
                    respond_to 
                } => {
                    let result = self.cache.read(&raw_digest);
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to read config schema digests: {:?}", e);
                    }
                },
                WorkerCommand::Write {
                    digests,
                    overwrite,
                    respond_to,
                } => {
                    let result = self.cache.write(digests, overwrite);
                    if let Err(e) = respond_to.send(result) {
                        error!("Actor failed to write the config schema digests: {:?}", e);
                    }
                }
            }
        }
    }
}

// This struct is the public interface for the cache. In practice, it just forwards
// commands to the actor and returns the results once they are ready.
#[derive(Clone)]
pub struct AsyncConfigSchemaDigestCache {
    sender: Sender<WorkerCommand>,
}

impl AsyncConfigSchemaDigestCache {
    pub fn spawn(dir: Dir) -> AsyncConfigSchemaDigestCache {
        let (sender, receiver) = mpsc::channel(32);
        let worker = Worker { cache: SyncConfigSchemaDigestCache::new(dir), receiver };

        tokio::spawn(worker.run());
        AsyncConfigSchemaDigestCache { sender }
    }

    pub async fn read_optional(&self, raw_digest: &str) -> Result<Option<ConfigSchemaDigests>, StorageErr> {
        let (send, recv) = oneshot::channel();
        self.sender.send(WorkerCommand::ReadOptional {
            raw_digest: raw_digest.to_string(),
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

    pub async fn read(&self, raw_digest: &str) -> Result<ConfigSchemaDigests, StorageErr> {
        let (send, recv) = oneshot::channel();
        self.sender.send(WorkerCommand::Read {
            raw_digest: raw_digest.to_string(),
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
        digests: ConfigSchemaDigests,
        overwrite: bool,
    ) -> Result<(), StorageErr> {
        let (send, recv) = oneshot::channel();
        self.sender.send(WorkerCommand::Write {
            digests,
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