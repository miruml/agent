// internal crates
use crate::errors::{MiruError, Trace};
use crate::filesys::{errors::FileSysErr, file::File};

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum StorageErr {
    // storage errors
    #[error("BackupFileReadErr: {source} {file} {backup_file}")]
    BackupFileReadErr {
        source: Box<FileSysErr>,
        file: File,
        backup_file: File,
        trace: Box<Trace>,
    },
    #[error("CacheElementNotFound: {msg}")]
    CacheElementNotFound {
        msg: String,
        trace: Box<Trace>,
    },
    #[error("Invalid Dir Name: {name}")]
    InvalidDirName {
        name: String,
        expected_name: Option<String>,
        trace: Box<Trace>,
    },
    #[error("Invalid File Name: {name}")]
    InvalidFileName {
        name: String,
        expected_name: Option<String>,
        trace: Box<Trace>,
    },
    #[error("Library Dir Not Found: {id}")]
    LibraryDirNotFound { id: String, trace: Box<Trace> },
    #[error("MissingScriptRunErr: {job_run_id:?}, {script_run_id:?}")]
    MissingScriptRunErr {
        job_run_id: String,
        script_run_id: String,
        trace: Box<Trace>,
    },
    #[error("MissingScriptRunsErr: {job_run_id:?}")]
    MissingScriptRunsErr {
        job_run_id: String,
        trace: Box<Trace>,
    },
    #[error("Regex Capture Error: {msg}")]
    RegexCaptureErr { msg: String, trace: Box<Trace> },

    // internal crate errors
    #[error("File System Error: {source}")]
    FileSysErr {
        source: FileSysErr,
        trace: Box<Trace>,
    },

    // external crate errors
    #[error("Parse Int Error: {source}")]
    ParseIntErr {
        source: std::num::ParseIntError,
        trace: Box<Trace>,
    },
    #[error("Send Actor Message Error: {source}")]
    SendActorMessageErr {
        source: Box<dyn std::error::Error + Send + Sync>,
        trace: Box<Trace>,
    },
    #[error("Receive Actor Message Error: {source}")]
    ReceiveActorMessageErr {
        source: Box<dyn std::error::Error + Send + Sync>,
        trace: Box<Trace>,
    },
}

impl MiruError for StorageErr {
    fn network_connection_error(&self) -> bool {
        false
    }
}
