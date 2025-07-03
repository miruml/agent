// standard library
use std::fmt;

// internal crates
use crate::errors::Trace;
use crate::errors::{Code, HTTPCode, MiruError};

// external crates
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

#[derive(Debug)]
pub struct PollErr {
    pub source: rumqttc::ConnectionError,
    pub trace: Box<Trace>,
}

impl MiruError for PollErr {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        match self.source {
            // network connection errors
            rumqttc::ConnectionError::NetworkTimeout => true,
            rumqttc::ConnectionError::FlushTimeout => true,
            rumqttc::ConnectionError::NotConnAck(_) => true,

            // non-network connection errors
            rumqttc::ConnectionError::ConnectionRefused(_) => false,
            _ => false,
        }
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for PollErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to poll event loop: {}", self.source)
    }
}

#[derive(Debug)]
pub struct TimeoutErr {
    pub msg: String,
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
        true
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for TimeoutErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Request timed out: {}", self.msg)
    }
}

#[derive(Debug)]
pub struct PublishErr {
    pub source: rumqttc::ClientError,
    pub trace: Box<Trace>,
}

impl MiruError for PublishErr {
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

impl fmt::Display for PublishErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to publish message: {}", self.source)
    }
}

#[derive(Debug)]
pub enum MQTTError {
    TimeoutErr(Box<TimeoutErr>),
    PollErr(Box<PollErr>),
    PublishErr(Box<PublishErr>),
}

macro_rules! forward_error_method {
    ($self:ident, $method:ident $(, $arg:expr)?) => {
        match $self {
            MQTTError::TimeoutErr(e) => e.$method($($arg)?),
            MQTTError::PollErr(e) => e.$method($($arg)?),
            MQTTError::PublishErr(e) => e.$method($($arg)?),
        }
    };
}

impl fmt::Display for MQTTError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        forward_error_method!(self, fmt, f)
    }
}

impl MiruError for MQTTError {
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
