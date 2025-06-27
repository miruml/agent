// standard crates
use std::fmt;

// internal crates
use crate::crud::errors::CrudErr;
use crate::errors::{Code, HTTPCode, MiruError, Trace};
use crate::http::errors::HTTPErr;
use crate::storage::errors::StorageErr;

#[derive(Debug)]
pub struct SyncCrudErr{
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
        write!(f, "Config instance data not found for config instance '{}'", self.instance_id)
    }
}


#[derive(Debug)]
pub enum SyncErr {
    CrudErr(SyncCrudErr),
    HTTPClientErr(SyncHTTPClientErr),
    StorageErr(SyncStorageErr),

    ConfigInstanceDataNotFound(ConfigInstanceDataNotFoundErr),
}

macro_rules! forward_error_method {
    ($self:ident, $method:ident $(, $arg:expr)?) => {
        match $self {
            SyncErr::CrudErr(e) => e.$method($($arg)?),
            SyncErr::HTTPClientErr(e) => e.$method($($arg)?),
            SyncErr::StorageErr(e) => e.$method($($arg)?),

            SyncErr::ConfigInstanceDataNotFound(e) => e.$method($($arg)?),
        }
    };
}

impl fmt::Display for SyncErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyncErr::CrudErr(e) => e.fmt(f),
            SyncErr::HTTPClientErr(e) => e.fmt(f),
            SyncErr::StorageErr(e) => e.fmt(f),

            SyncErr::ConfigInstanceDataNotFound(e) => e.fmt(f),
        }
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