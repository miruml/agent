// standard library 
use std::fmt;

// internal crates
use crate::errors::{MiruError, Code, HTTPCode};
use crate::errors::Trace;
use openapi_client::models::ErrorResponse;
use crate::http::client::RequestContext;

// external crates
use std::time::Duration;
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};



#[derive(Debug)]
pub struct RequestFailed {
    pub request: RequestContext,
    pub status: reqwest::StatusCode,
    pub error: Option<ErrorResponse>,
    pub trace: Box<Trace>,
}

impl MiruError for RequestFailed {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        self.status
    }

    fn is_network_connection_error(&self) -> bool {
        false
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for RequestFailed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let debug_msg = match &self.error {
            Some(error) => error.error.debug_message.clone(),
            None => "unknown miru server error".to_string(),
        };
        write!(
            f, 
            "Request {} failed with status code {}: {}",
            self.request,
            self.status,
            debug_msg
        )
    }
}



#[derive(Debug)]
pub struct TimeoutErr {
    pub msg: String,
    pub request: RequestContext,
    pub timeout: Duration,
    pub trace: Box<Trace>,
}

impl MiruError for TimeoutErr {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        false
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for TimeoutErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Request {} timed out after {} seconds", self.request, self.timeout.as_secs())
    }
}

#[derive(Debug)]
pub struct CacheErr {
    pub is_network_connection_error: bool,
    pub msg: String,
    pub trace: Box<Trace>,
}

impl MiruError for CacheErr {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        self.is_network_connection_error
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for CacheErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Request returned a cache error: {}", self.msg)
    }
}

#[derive(Debug)]
pub struct ConnectionErr {
    pub source: reqwest::Error,
    pub trace: Box<Trace>,
}

impl MiruError for ConnectionErr {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        true
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for ConnectionErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Network connection error: {}", self.source)
    }
}

#[derive(Debug)]
pub struct DecodeRespBodyErr {
    pub source: reqwest::Error,
    pub trace: Box<Trace>,
}

impl MiruError for DecodeRespBodyErr {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        false
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for DecodeRespBodyErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unable to decode response body: {}", self.source)
    }
}

#[derive(Debug)]
pub struct InvalidHeaderValueErr {
    pub msg: String,
    pub source: reqwest::header::InvalidHeaderValue,
    pub trace: Box<Trace>,
}

impl MiruError for InvalidHeaderValueErr {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        false
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for InvalidHeaderValueErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid header value: {}", self.source)
    }
}

#[derive(Debug)]
pub struct ParseJSONErr {
    pub source: serde_json::Error,
    pub trace: Box<Trace>,
}

impl MiruError for ParseJSONErr {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        false
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for ParseJSONErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unable to parse JSON: {}", self.source)
    }
}

#[derive(Debug)]
pub struct ReqwestErr {
    pub source: reqwest::Error,
    pub trace: Box<Trace>,
}

impl MiruError for ReqwestErr {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        false
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for ReqwestErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Reqwest crate error: {}", self.source)
    }
}

#[derive(Debug)]
pub struct MockErr {
    pub is_network_connection_error: bool,
    pub trace: Box<Trace>,
}

impl MiruError for MockErr {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        self.is_network_connection_error
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for MockErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Mock error (is network connection error: {})", self.is_network_connection_error)
    }
}


#[derive(Debug)]
pub enum HTTPErr {
    // HTTP errors
    RequestFailed(RequestFailed),
    TimeoutErr(TimeoutErr),
    CacheErr(CacheErr),

    // external crate errors
    ConnectionErr(ConnectionErr),
    DecodeRespBodyErr(DecodeRespBodyErr),
    InvalidHeaderValueErr(InvalidHeaderValueErr),
    ParseJSONErr(ParseJSONErr),
    ReqwestErr(ReqwestErr),

    // mock errors (not for production use)
    MockErr(MockErr),
}

macro_rules! forward_error_method {
    ($self:ident, $method:ident $(, $arg:expr)?) => {
        match $self {
            Self::RequestFailed(e) => e.$method($($arg)?),
            Self::TimeoutErr(e) => e.$method($($arg)?),
            Self::CacheErr(e) => e.$method($($arg)?),
            Self::ConnectionErr(e) => e.$method($($arg)?),
            Self::DecodeRespBodyErr(e) => e.$method($($arg)?),
            Self::InvalidHeaderValueErr(e) => e.$method($($arg)?),
            Self::ParseJSONErr(e) => e.$method($($arg)?),
            Self::ReqwestErr(e) => e.$method($($arg)?),
            Self::MockErr(e) => e.$method($($arg)?),
        }
    };
}

impl fmt::Display for HTTPErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        forward_error_method!(self, fmt, f)
    }
}

impl MiruError for HTTPErr {
    fn code(&self) -> Code {
        forward_error_method!(self, code)
    }

    fn http_status(&self) -> HTTPCode {
        forward_error_method!(self, http_status)
    }

    fn is_network_connection_error(&self) -> bool {
        forward_error_method!(self, is_network_connection_error)
    }

    fn params(&self) -> Option<serde_json::Value> {
        forward_error_method!(self, params)
    }
}

pub fn reqwest_err_to_http_client_err(e: reqwest::Error, trace: Box<Trace>) -> HTTPErr {
    if e.is_connect() {
        HTTPErr::ConnectionErr(ConnectionErr { source: e, trace })
    } else if e.is_decode() {
        HTTPErr::DecodeRespBodyErr(DecodeRespBodyErr { source: e, trace })
    } else {
        HTTPErr::ReqwestErr(ReqwestErr { source: e, trace })
    }
}
