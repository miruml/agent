// standard library
use std::fmt;

// internal crates
use crate::auth::errors::AuthErr;
use crate::crypt::errors::CryptErr;
use crate::errors::MiruError;
use crate::errors::{Code, HTTPCode, Trace};
use crate::filesys::errors::FileSysErr;
use crate::http::errors::HTTPErr;
use crate::storage::errors::StorageErr;


#[derive(Debug)]
pub struct ServerAuthErr {
    pub source: AuthErr,
    pub trace: Box<Trace>,
}

impl MiruError for ServerAuthErr {
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

impl fmt::Display for ServerAuthErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "server auth error: {}", self.source)
    }
}

#[derive(Debug)]
pub struct ServerCryptErr {
    pub source: CryptErr,
    pub trace: Box<Trace>,
}

impl MiruError for ServerCryptErr {
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

impl fmt::Display for ServerCryptErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "server crypt error: {}", self.source)
    }
}

#[derive(Debug)]
pub struct ServerFileSysErr {
    pub source: FileSysErr,
    pub trace: Box<Trace>,
}

impl MiruError for ServerFileSysErr {
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

impl fmt::Display for ServerFileSysErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "server file system error: {}", self.source)
    }
}

#[derive(Debug)]
pub struct ServerHTTPErr {
    pub source: HTTPErr,
    pub trace: Box<Trace>,
}

impl MiruError for ServerHTTPErr {
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

impl fmt::Display for ServerHTTPErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "http client error: {}", self.source)
    }
}

#[derive(Debug)]
pub struct ServerStorageErr {
    pub source: StorageErr,
    pub trace: Box<Trace>,
}

impl MiruError for ServerStorageErr {
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

impl fmt::Display for ServerStorageErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "server storage error: {}", self.source)
    }
}

#[derive(Debug)]
pub struct IOErr {
    pub source: std::io::Error,
    pub trace: Box<Trace>,
}

impl MiruError for IOErr {
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

impl fmt::Display for IOErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "server io error: {}", self.source)
    }
}

#[derive(Debug)]
pub struct JoinHandleErr {
    pub source: Box<dyn std::error::Error + Send + Sync>,
    pub trace: Box<Trace>,
}

impl MiruError for JoinHandleErr {
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

impl fmt::Display for JoinHandleErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "join handle error: {}", self.source)
    }
}

#[derive(Debug)]
pub struct SendShutdownSignalErr {
    pub service: String,
    pub trace: Box<Trace>,
}

impl MiruError for SendShutdownSignalErr {
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

impl fmt::Display for SendShutdownSignalErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to send shutdown signal to {}", self.service)
    }
}


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
pub enum ServerErr {
    // server errors
    TimestampConversionErr(TimestampConversionErr),

    // internal crate errors
    AuthErr(ServerAuthErr),
    CryptErr(ServerCryptErr),
    FileSysErr(ServerFileSysErr),
    HTTPErr(ServerHTTPErr),
    StorageErr(ServerStorageErr),

    // external crate errors
    IOErr(IOErr),
    SendShutdownSignalErr(SendShutdownSignalErr),
    JoinHandleErr(JoinHandleErr),
}

macro_rules! forward_error_method {
    ($self:ident, $method:ident $(, $arg:expr)?) => {
        match $self {
            Self::TimestampConversionErr(e) => e.$method($($arg)?),
            Self::AuthErr(e) => e.$method($($arg)?),
            Self::CryptErr(e) => e.$method($($arg)?),
            Self::FileSysErr(e) => e.$method($($arg)?),
            Self::HTTPErr(e) => e.$method($($arg)?),
            Self::StorageErr(e) => e.$method($($arg)?),
            Self::IOErr(e) => e.$method($($arg)?),
            Self::SendShutdownSignalErr(e) => e.$method($($arg)?),
            Self::JoinHandleErr(e) => e.$method($($arg)?),
        }
    };
}

impl fmt::Display for ServerErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        forward_error_method!(self, fmt, f)
    }
}

impl MiruError for ServerErr {
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