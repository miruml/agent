// standard library
use std::path::PathBuf;
// internal crates
use crate::errors::{MiruError, Trace};
use crate::filesys::{dir::Dir, file::File};

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum FileSysErr {
    // FileSys errors
    #[error("BadFileExtensionErr: {expected:?} != {actual:?} for file: {file}")]
    BadFileExtensionErr {
        expected: String,
        actual: String,
        file: File,
        trace: Box<Trace>,
    },
    #[error("BadFilePathExprErr: {expected:?} != {actual:?} for file: {file}")]
    BadFilePathExprErr {
        expected: String,
        actual: String,
        file: File,
        trace: Box<Trace>,
    },
    #[error("CreateHomeDirErr: home directory not found")]
    CreateHomeDirErr,
    #[error("FileNotFound: {file}")]
    FileNotFound { file: File, trace: Box<Trace> },
    #[error("Invalid Dir Name: {name}")]
    InvalidDirNameErr { name: String, trace: Box<Trace> },
    #[error("NoDirNameErr: {dir}")]
    NoDirNameErr { dir: Dir, trace: Box<Trace> },
    #[error("NoFileNameErr: {file}")]
    NoFileNameErr { file: File, trace: Box<Trace> },
    #[error("PathDoesNotExist: {path}")]
    PathDoesNotExist { path: PathBuf, trace: Box<Trace> },
    #[error("PathExists: {path}")]
    PathExists { path: PathBuf, trace: Box<Trace> },
    #[error("ReadDirErr: {source}")]
    ReadDirErr {
        source: std::io::Error,
        dir: Dir,
        trace: Box<Trace>,
    },
    #[error("UnknownParentDirErr: {dir}")]
    UnknownDirParentDirErr { dir: Dir, trace: Box<Trace> },
    #[error("UnknownParentDirErr: {file}")]
    UnknownFileParentDirErr { file: File, trace: Box<Trace> },
    #[error("UnknownCurrentDirErr: {source}")]
    UnknownCurrentDirErr { source: std::io::Error, trace: Box<Trace> },

    // internal crate errors

    // external crate errors
    #[error("AtomicWriteFileErr: {source}")]
    AtomicWriteFileErr {
        source: std::io::Error,
        file: File,
        trace: Box<Trace>,
    },
    #[error("ConvertUTF8Err: {source}")]
    ConvertUTF8Err {
        source: std::str::Utf8Error,
        trace: Box<Trace>,
    },
    #[error("CopyFileErr: {source}")]
    CopyFileErr {
        source: std::io::Error,
        src_file: File,
        dest_file: File,
        trace: Box<Trace>,
    },
    #[error("CreateDirErr: {source}")]
    CreateDirErr {
        source: std::io::Error,
        dir: Dir,
        trace: Box<Trace>,
    },
    #[error("CreateSymlinkErr: {source}")]
    CreateSymlinkErr {
        source: std::io::Error,
        file: File,
        link: File,
        trace: Box<Trace>,
    },
    #[error("CreateTempDirErr: {source}")]
    CreateTmpDirErr {
        source: std::io::Error,
        trace: Box<Trace>,
    },
    #[error("CreateTmpFileErr: {source}")]
    CreateTmpFileErr {
        source: std::io::Error,
        trace: Box<Trace>,
    },
    #[error("DeleteDirErr: {source}")]
    DeleteDirErr {
        source: std::io::Error,
        dir: Dir,
        trace: Box<Trace>,
    },
    #[error("DeleteFileErr: {source}")]
    DeleteFileErr {
        source: std::io::Error,
        file: File,
        trace: Box<Trace>,
    },
    #[error("FileMetaDataErr: {source}")]
    FileMetaDataErr {
        source: std::io::Error,
        trace: Box<Trace>,
    },
    #[error("MoveDirErr: {source}")]
    MoveDirErr {
        source: std::io::Error,
        src_dir: Dir,
        dest_dir: Dir,
        trace: Box<Trace>,
    },
    #[error("MoveFileErr: {source}")]
    MoveFileErr {
        source: std::io::Error,
        src_file: File,
        dest_file: File,
        trace: Box<Trace>,
    },
    #[error("OpenFileErr: {source}")]
    OpenFileErr {
        source: std::io::Error,
        file: File,
        trace: Box<Trace>,
    },
    #[error("ParseJSONErr: {source}")]
    ParseJSONErr {
        source: serde_json::Error,
        file: File,
        trace: Box<Trace>,
    },
    #[error("ReadFileErr: {source}")]
    ReadFileErr {
        source: std::io::Error,
        file: File,
        trace: Box<Trace>,
    },
    #[error("StreamBytesErr: {source}")]
    StreamBytesErr {
        source: reqwest::Error,
        trace: Box<Trace>,
    },
    #[error("SystemTimeErr: {source}")]
    SystemTimeErr {
        source: std::time::SystemTimeError,
        trace: Box<Trace>,
    },
    #[error("WriteFileErr: {source}")]
    WriteFileErr {
        source: std::io::Error,
        file: File,
        trace: Box<Trace>,
    },
}

impl MiruError for FileSysErr {
    fn network_connection_error(&self) -> bool {
        false
    }
}
