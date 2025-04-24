// standard library
use std::fmt;

// internal crates
use crate::crypt::errors::CryptErr;
use crate::errors::{Code, HTTPCode, MiruError, Trace};
use crate::filesys::errors::FileSysErr;
use crate::http::errors::HTTPErr;

#[derive(Debug)]
pub struct TimestampConversionErr {
    pub msg: String,
    pub trace: Box<Trace>,
}

impl MiruError for TimestampConversionErr {
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

impl fmt::Display for TimestampConversionErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "timestamp conversion error: {}", self.msg)
    }
}

#[derive(Debug)]
pub struct AuthCryptErr {
    pub source: CryptErr,
    pub trace: Box<Trace>,
}

impl MiruError for AuthCryptErr {
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

impl fmt::Display for AuthCryptErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cryptography error: {}", self.source)
    }
}

#[derive(Debug)]
pub struct AuthFileSysErr {
    pub source: FileSysErr,
    pub trace: Box<Trace>,
}

impl MiruError for AuthFileSysErr {
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

impl fmt::Display for AuthFileSysErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "file system error: {}", self.source)
    }
}

#[derive(Debug)]
pub struct AuthHTTPErr {
    pub source: HTTPErr,
    pub trace: Box<Trace>,
}

impl MiruError for AuthHTTPErr {
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

impl fmt::Display for AuthHTTPErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "http client error: {}", self.source)
    }
}

#[derive(Debug)]
pub struct SerdeErr {
    pub source: serde_json::Error,
    pub trace: Box<Trace>,
}

impl MiruError for SerdeErr {
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

impl fmt::Display for SerdeErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "serialization error: {}", self.source)
    }
}

#[derive(Debug)]
pub struct SendActorMessageErr {
    pub source: Box<dyn std::error::Error + Send + Sync>,
    pub trace: Box<Trace>,
}

impl MiruError for SendActorMessageErr {
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

impl fmt::Display for SendActorMessageErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to send actor message: {}", self.source)
    }
}

#[derive(Debug)]
pub struct ReceiveActorMessageErr {
    pub source: Box<dyn std::error::Error + Send + Sync>,
    pub trace: Box<Trace>,
}

impl MiruError for ReceiveActorMessageErr {
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

impl fmt::Display for ReceiveActorMessageErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to receive actor message: {}", self.source)
    }
}

#[derive(Debug)]
pub enum AuthErr {
    // auth errors
    TimestampConversionErr(TimestampConversionErr),
    
    // internal crate errors
    CryptErr(AuthCryptErr),
    FileSysErr(AuthFileSysErr),
    HTTPErr(AuthHTTPErr),

    // external crate errors
    SerdeErr(SerdeErr),
    SendActorMessageErr(SendActorMessageErr),
    ReceiveActorMessageErr(ReceiveActorMessageErr),
}

macro_rules! forward_error_method {
    ($self:ident, $method:ident $(, $arg:expr)?) => {
        match $self {
            Self::CryptErr(e) => e.$method($($arg)?),
            Self::SerdeErr(e) => e.$method($($arg)?),
            Self::FileSysErr(e) => e.$method($($arg)?),
            Self::HTTPErr(e) => e.$method($($arg)?),
            Self::TimestampConversionErr(e) => e.$method($($arg)?),
            Self::SendActorMessageErr(e) => e.$method($($arg)?),
            Self::ReceiveActorMessageErr(e) => e.$method($($arg)?),
        }
    };
}

impl fmt::Display for AuthErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        forward_error_method!(self, fmt, f)
    }
}

impl MiruError for AuthErr {
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