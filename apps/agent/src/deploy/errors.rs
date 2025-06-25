// standard crates
use std::fmt;

// internal crates
use crate::errors::{Code, HTTPCode, MiruError, Trace};
use crate::filesys::errors::FileSysErr;
use crate::models::config_instance::{
    ConfigInstance, ConfigInstanceActivityStatus, ConfigInstanceErrorStatus,
    ConfigInstanceTargetStatus,
};
use crate::storage::errors::StorageErr;

#[derive(Debug)]
pub struct ConflictingDeploymentsErr {
    pub trace: Box<Trace>,
}

impl MiruError for ConflictingDeploymentsErr {
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

impl fmt::Display for ConflictingDeploymentsErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FIXME")
    }
}

#[derive(Debug)]
pub struct ConfigInstanceWithMismatchingFilepath {
    pub id: String,
    pub filepath: Option<String>,
    pub target_status: ConfigInstanceTargetStatus,
    pub activity_status: ConfigInstanceActivityStatus,
    pub error_status: ConfigInstanceErrorStatus,
}

impl ConfigInstanceWithMismatchingFilepath {
    pub fn from_instance(instance: ConfigInstance) -> Self {
        Self {
            id: instance.id,
            filepath: instance.filepath,
            target_status: instance.target_status,
            activity_status: instance.activity_status,
            error_status: instance.error_status,
        }
    }
}

#[derive(Debug)]
pub struct MismatchingFilepathErr {
    pub old: ConfigInstanceWithMismatchingFilepath,
    pub new: ConfigInstanceWithMismatchingFilepath,
    pub trace: Box<Trace>,
}

impl MiruError for MismatchingFilepathErr {
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

impl fmt::Display for MismatchingFilepathErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Expected old and new config instances to have the same filepath, but got {:?} (instance {:?}) and {:?} (instance {:?})", self.old.filepath, self.old, self.new.filepath, self.new)
    }
}

#[derive(Debug)]
pub struct DeployFileSysErr {
    pub source: FileSysErr,
    pub trace: Box<Trace>,
}

impl MiruError for DeployFileSysErr {
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

impl fmt::Display for DeployFileSysErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "file system error: {}", self.source)
    }
}

#[derive(Debug)]
pub struct DeployStorageErr {
    pub source: StorageErr,
    pub trace: Box<Trace>,
}

impl MiruError for DeployStorageErr {
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

impl fmt::Display for DeployStorageErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "storage error: {}", self.source)
    }
}

#[derive(Debug)]
pub enum DeployErr {
    ConflictingDeploymentsErr(ConflictingDeploymentsErr),
    DeployFileSysErr(DeployFileSysErr),
    DeployStorageErr(DeployStorageErr),
    MismatchingFilepathErr(MismatchingFilepathErr),
}

macro_rules! forward_error_method {
    ($self:ident, $method:ident $(, $arg:expr)?) => {
        match $self {
            DeployErr::ConflictingDeploymentsErr(e) => e.$method($($arg)?),
            DeployErr::DeployFileSysErr(e) => e.$method($($arg)?),
            DeployErr::DeployStorageErr(e) => e.$method($($arg)?),
            DeployErr::MismatchingFilepathErr(e) => e.$method($($arg)?),
        }
    };
}

impl fmt::Display for DeployErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        forward_error_method!(self, fmt, f)
    }
}

impl MiruError for DeployErr {
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
