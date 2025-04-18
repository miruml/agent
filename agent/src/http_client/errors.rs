// internal crates
use crate::errors::MiruError;
use crate::errors::Trace;
use openapi_client::models::ErrorResponse;

// external crates
use std::time::Duration;
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum HTTPErr {
    // HTTP errors
    #[error("Response Body Missing Error: {msg}")]
    ResponseBodyMissingErr { msg: String, trace: Box<Trace> },
    #[error("Response Failed: {url:?} {status:?} {error:?}")]
    ResponseFailed {
        status: reqwest::StatusCode,
        url: String,
        error: Option<ErrorResponse>,
        trace: Box<Trace>,
    },
    #[error("Timeout Error: {msg}")]
    TimeoutErr {
        msg: String,
        timeout: Duration,
        trace: Box<Trace>,
    },
    #[error("Cache Error: {msg}")]
    CacheErr {
        is_network_connection_error: bool,
        msg: String,
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
    #[error("Parse JSON Error: {source}")]
    ParseJSONErr {
        source: serde_json::Error,
        trace: Box<Trace>,
    },
    #[error("Reqwest Error: {source}")]
    ReqwestErr {
        source: reqwest::Error,
        trace: Box<Trace>,
    },

    // mock errors (not for production use)
    #[error("Mock Error: {is_network_connection_error}")]
    MockErr {
        is_network_connection_error: bool,
        trace: Box<Trace>,
    },
}

impl AsRef<dyn MiruError> for HTTPErr {
    fn as_ref(&self) -> &(dyn MiruError + 'static) {
        self
    }
}

impl MiruError for HTTPErr {
    fn is_network_connection_error(&self) -> bool {
        if let HTTPErr::CacheErr { is_network_connection_error, .. } = self {
            *is_network_connection_error
        } else if let HTTPErr::MockErr { is_network_connection_error, .. } = self {
            *is_network_connection_error
        } else {
            matches!(
                self,
                HTTPErr::ConnectionErr { .. } | HTTPErr::TimeoutErr { .. }
            )
        }
    }
}

pub fn reqwest_err_to_http_client_err(e: reqwest::Error, trace: Box<Trace>) -> HTTPErr {
    if e.is_connect() {
        HTTPErr::ConnectionErr { source: e, trace }
    } else if e.is_decode() {
        HTTPErr::DecodeRespBodyErr { source: e, trace }
    } else {
        HTTPErr::ReqwestErr { source: e, trace }
    }
}
