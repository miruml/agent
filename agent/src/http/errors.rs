// internal crates
use crate::errors::MiruError;
use crate::errors::Trace;
use crate::filesys::file::File;
use crate::http::client::APIResponse;
use crate::http::client::Code;
// external crates
use std::time::Duration;
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum HTTPErr {
    // HTTP errors
    #[error("File Not Found: {msg}")]
    FileNotFound { msg: String, trace: Box<Trace> },
    #[error("Response Data Missing Error: {msg}")]
    ResponseDataMissingErr { msg: String, trace: Box<Trace> },
    #[error("Response Error: {http_code:?} {resp:?}")]
    ResponseErr {
        http_code: reqwest::StatusCode,
        resp: reqwest::Response,
        trace: Box<Trace>,
    },
    #[error("Miru Response Error: {http_code:?} {code:?} {msg}")]
    MiruResponseErr {
        http_code: reqwest::StatusCode,
        code: Code,
        msg: String,
        trace: Box<Trace>,
    },
    #[error("Timeout Error: {msg}")]
    TimeoutErr {
        msg: String,
        timeout: Duration,
        trace: Box<Trace>,
    },

    // internal crate errors
    #[error("Crypt Error: {source}")]
    AuthErr {
        source: crate::auth::errors::AuthErr,
        trace: Box<Trace>,
    },
    #[error("File System Error: {source}")]
    FileSysErr {
        source: crate::filesys::errors::FileSysErr,
        trace: Box<Trace>,
    },
    #[error("Storage Error: {source}")]
    StorageErr {
        source: crate::storage::errors::StorageErr,
        trace: Box<Trace>,
    },

    // external crate errors
    #[error("Connection Error: {source}")]
    ConnectionErr {
        source: reqwest::Error,
        trace: Box<Trace>,
    },
    #[error("Decode Response Body Error: {source}")]
    DecodeRespBodyErr {
        source: reqwest::Error,
        trace: Box<Trace>,
    },
    #[error("Invalid Header Value Error: {msg}: {source}")]
    InvalidHeaderValueErr {
        msg: String,
        source: reqwest::header::InvalidHeaderValue,
        trace: Box<Trace>,
    },
    #[error("Mutex Error: {msg}")]
    MutexErr { msg: String, trace: Box<Trace> },
    #[error("OpenFileErr: {source}")]
    OpenFileErr {
        source: std::io::Error,
        file: File,
        trace: Box<Trace>,
    },
    #[error("Parse JSON Error: {source}")]
    ParseJSONErr {
        source: serde_json::Error,
        trace: Box<Trace>,
    },
    #[error("Parse API Response Error: {source}")]
    ParseAPIResponseErr {
        api_resp: APIResponse,
        source: serde_json::Error,
        trace: Box<Trace>,
    },
    #[error("Request Error: {source}")]
    RequestErr {
        source: reqwest::Error,
        trace: Box<Trace>,
    },
    #[error("StreamBytesErr: {source}")]
    StreamBytesErr {
        source: reqwest::Error,
        trace: Box<Trace>,
    },
    #[error("WriteFileErr: {source}")]
    WriteFileErr {
        source: std::io::Error,
        file: File,
        trace: Box<Trace>,
    },
}

impl AsRef<dyn MiruError> for HTTPErr {
    fn as_ref(&self) -> &(dyn MiruError + 'static) {
        self
    }
}

impl HTTPErr {
    pub fn is_error_type(&self, c: Code) -> bool {
        matches!(self, HTTPErr::MiruResponseErr { code, .. } if *code == c)
    }
}

impl MiruError for HTTPErr {
    fn is_poor_signal_error(&self) -> bool {
        matches!(
            self,
            HTTPErr::ConnectionErr { .. } | HTTPErr::TimeoutErr { .. }
        )
    }
}

pub fn reqwest_err_to_http_client_err(e: reqwest::Error, trace: Box<Trace>) -> HTTPErr {
    if e.is_connect() {
        HTTPErr::ConnectionErr { source: e, trace }
    } else if e.is_decode() {
        HTTPErr::DecodeRespBodyErr { source: e, trace }
    } else {
        HTTPErr::RequestErr { source: e, trace }
    }
}
