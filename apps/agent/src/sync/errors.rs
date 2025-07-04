// standard crates
use std::fmt;

// internal crates
use crate::auth::errors::AuthErr;
use crate::cache::errors::CacheErr;
use crate::crud::errors::CrudErr;
use crate::deploy::errors::DeployErr;
use crate::errors::{Code, HTTPCode, MiruError, Trace};
use crate::http::errors::HTTPErr;
use crate::storage::errors::StorageErr;

#[derive(Debug)]
pub struct SyncAuthErr {
    pub source: AuthErr,
    pub trace: Box<Trace>,
}

impl MiruError for SyncAuthErr {
    fn code(&self) -> Code {
        self.source.code()
    }

    fn http_status(&self) -> HTTPCode {
        self.source.http_status()
    }

    fn is_network_connection_error(&self) -> bool {
        self.source.is_network_connection_error()
    }

    fn params(&self) -> Option<serde_json::Value> {
        self.source.params()
    }
}

impl fmt::Display for SyncAuthErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Auth error: {}", self.source)
    }
}

#[derive(Debug)]
pub struct SyncCacheErr {
    pub source: CacheErr,
    pub trace: Box<Trace>,
}

impl MiruError for SyncCacheErr {
    fn code(&self) -> Code {
        self.source.code()
    }

    fn http_status(&self) -> HTTPCode {
        self.source.http_status()
    }

    fn is_network_connection_error(&self) -> bool {
        self.source.is_network_connection_error()
    }

    fn params(&self) -> Option<serde_json::Value> {
        self.source.params()
    }
}

impl fmt::Display for SyncCacheErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cache error: {}", self.source)
    }
}

#[derive(Debug)]
pub struct SyncCrudErr {
    pub source: CrudErr,
    pub trace: Box<Trace>,
}

impl MiruError for SyncCrudErr {
    fn code(&self) -> Code {
        self.source.code()
    }

    fn http_status(&self) -> HTTPCode {
        self.source.http_status()
    }

    fn is_network_connection_error(&self) -> bool {
        self.source.is_network_connection_error()
    }

    fn params(&self) -> Option<serde_json::Value> {
        self.source.params()
    }
}

impl fmt::Display for SyncCrudErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Crud error: {}", self.source)
    }
}

#[derive(Debug)]
pub struct SyncDeployErr {
    pub source: DeployErr,
    pub trace: Box<Trace>,
}

impl MiruError for SyncDeployErr {
    fn code(&self) -> Code {
        self.source.code()
    }

    fn http_status(&self) -> HTTPCode {
        self.source.http_status()
    }

    fn is_network_connection_error(&self) -> bool {
        self.source.is_network_connection_error()
    }

    fn params(&self) -> Option<serde_json::Value> {
        self.source.params()
    }
}

impl fmt::Display for SyncDeployErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Deploy error: {}", self.source)
    }
}

#[derive(Debug)]
pub struct SyncHTTPClientErr {
    pub source: HTTPErr,
    pub trace: Box<Trace>,
}

impl MiruError for SyncHTTPClientErr {
    fn code(&self) -> Code {
        self.source.code()
    }

    fn http_status(&self) -> HTTPCode {
        self.source.http_status()
    }

    fn is_network_connection_error(&self) -> bool {
        self.source.is_network_connection_error()
    }

    fn params(&self) -> Option<serde_json::Value> {
        self.source.params()
    }
}

impl fmt::Display for SyncHTTPClientErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HTTP client error: {}", self.source)
    }
}

#[derive(Debug)]
pub struct SyncStorageErr {
    pub source: StorageErr,
    pub trace: Box<Trace>,
}

impl MiruError for SyncStorageErr {
    fn code(&self) -> Code {
        self.source.code()
    }

    fn http_status(&self) -> HTTPCode {
        self.source.http_status()
    }

    fn is_network_connection_error(&self) -> bool {
        self.source.is_network_connection_error()
    }

    fn params(&self) -> Option<serde_json::Value> {
        self.source.params()
    }
}

impl fmt::Display for SyncStorageErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Storage error: {}", self.source)
    }
}

#[derive(Debug)]
pub struct SyncErrors {
    pub source: Vec<SyncErr>,
    pub trace: Box<Trace>,
}

impl MiruError for SyncErrors {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        // is only a network connection error if all errors are network connection
        // errors
        for err in self.source.iter() {
            if !err.is_network_connection_error() {
                return false;
            }
        }
        true
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for SyncErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Sync error: {:?}", self.source)
    }
}

#[derive(Debug)]
pub struct ConfigInstanceDataNotFoundErr {
    pub instance_id: String,
    pub trace: Box<Trace>,
}

impl MiruError for ConfigInstanceDataNotFoundErr {
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

impl fmt::Display for ConfigInstanceDataNotFoundErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Config instance data not found for config instance '{}'",
            self.instance_id
        )
    }
}

pub type SendActorMessageErr = crate::cache::errors::SendActorMessageErr;
pub type ReceiveActorMessageErr = crate::cache::errors::ReceiveActorMessageErr;

#[derive(Debug)]
pub enum SyncErr {
    AuthErr(Box<SyncAuthErr>),
    CacheErr(Box<SyncCacheErr>),
    CrudErr(Box<SyncCrudErr>),
    DeployErr(Box<SyncDeployErr>),
    HTTPClientErr(Box<SyncHTTPClientErr>),
    StorageErr(Box<SyncStorageErr>),
    SyncErrors(Box<SyncErrors>),

    ConfigInstanceDataNotFound(Box<ConfigInstanceDataNotFoundErr>),
    SendActorMessageErr(Box<SendActorMessageErr>),
    ReceiveActorMessageErr(Box<ReceiveActorMessageErr>),
}

macro_rules! forward_error_method {
    ($self:ident, $method:ident $(, $arg:expr)?) => {
        match $self {
            SyncErr::AuthErr(e) => e.$method($($arg)?),
            SyncErr::CacheErr(e) => e.$method($($arg)?),
            SyncErr::CrudErr(e) => e.$method($($arg)?),
            SyncErr::DeployErr(e) => e.$method($($arg)?),
            SyncErr::HTTPClientErr(e) => e.$method($($arg)?),
            SyncErr::StorageErr(e) => e.$method($($arg)?),
            SyncErr::SyncErrors(e) => e.$method($($arg)?),

            SyncErr::SendActorMessageErr(e) => e.$method($($arg)?),
            SyncErr::ReceiveActorMessageErr(e) => e.$method($($arg)?),

            SyncErr::ConfigInstanceDataNotFound(e) => e.$method($($arg)?),
        }
    };
}

impl fmt::Display for SyncErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        forward_error_method!(self, fmt, f)
    }
}

impl MiruError for SyncErr {
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
