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
    #[error("Response Error: {http_code:?} {error:?}")]
    ResponseErr {
        http_code: reqwest::StatusCode,
        error: ErrorResponse,
        trace: Box<Trace>,
    },
    #[error("Timeout Error: {msg}")]
    TimeoutErr {
        msg: String,
        timeout: Duration,
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
    #[error("Request Error: {source}")]
    RequestErr {
        source: reqwest::Error,
        trace: Box<Trace>,
    },
}

impl AsRef<dyn MiruError> for HTTPErr {
    fn as_ref(&self) -> &(dyn MiruError + 'static) {
        self
    }
}

impl MiruError for HTTPErr {
    fn network_connection_error(&self) -> bool {
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
