// standard library
use std::fmt;

// internal crates
use config_agent::auth::errors::AuthErr;
use config_agent::crypt::errors::CryptErr;
use config_agent::errors::{Code, HTTPCode, MiruError, Trace};
use config_agent::filesys::errors::FileSysErr;
use config_agent::http::errors::HTTPErr;
use config_agent::storage::errors::StorageErr;

#[derive(Debug)]
pub struct InstallerAuthErr {
    pub source: AuthErr,
    pub trace: Box<Trace>,
}

impl MiruError for InstallerAuthErr {
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

impl fmt::Display for InstallerAuthErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.source)
    }
}

#[derive(Debug)]
pub struct InstallerCryptErr {
    pub source: CryptErr,
    pub trace: Box<Trace>,
}

impl MiruError for InstallerCryptErr {
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

impl fmt::Display for InstallerCryptErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.source)
    }
}

#[derive(Debug)]
pub struct InstallerFileSysErr {
    pub source: FileSysErr,
    pub trace: Box<Trace>,
}

impl MiruError for InstallerFileSysErr {
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

impl fmt::Display for InstallerFileSysErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.source)
    }
}

#[derive(Debug)]
pub struct InstallerHTTPErr {
    pub source: HTTPErr,
    pub trace: Box<Trace>,
}

impl MiruError for InstallerHTTPErr {
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

impl fmt::Display for InstallerHTTPErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.source)
    }
}

#[derive(Debug)]
pub struct InstallerStorageErr {
    pub source: StorageErr,
    pub trace: Box<Trace>,
}

impl MiruError for InstallerStorageErr {
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

impl fmt::Display for InstallerStorageErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.source)
    }
}

#[derive(Debug)]
pub struct DialoguerErr {
    pub source: dialoguer::Error,
    pub trace: Box<Trace>,
}

impl MiruError for DialoguerErr {
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

impl fmt::Display for DialoguerErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.source)
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
        write!(f, "{}", self.source)
    }
}

#[derive(Debug)]
pub struct ExecShellErr {
    pub msg: String,
    pub trace: Box<Trace>,
}

impl MiruError for ExecShellErr {
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

impl fmt::Display for ExecShellErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

#[derive(Debug)]
pub enum InstallerErr {
    // internal crate errors
    AuthErr(InstallerAuthErr),
    CryptErr(InstallerCryptErr),
    FileSysErr(InstallerFileSysErr),
    HTTPErr(InstallerHTTPErr),
    StorageErr(InstallerStorageErr),

    // external crate errors
    DialoguerErr(DialoguerErr),
    IOErr(IOErr),
    ExecShellErr(ExecShellErr),
}

macro_rules! forward_error_method {
    ($self:ident, $method:ident $(, $arg:expr)?) => {
        match $self {
            Self::AuthErr(e) => e.$method($($arg)?),
            Self::CryptErr(e) => e.$method($($arg)?),
            Self::FileSysErr(e) => e.$method($($arg)?),
            Self::HTTPErr(e) => e.$method($($arg)?),
            Self::StorageErr(e) => e.$method($($arg)?),
            Self::DialoguerErr(e) => e.$method($($arg)?),
            Self::IOErr(e) => e.$method($($arg)?),
            Self::ExecShellErr(e) => e.$method($($arg)?),
        }
    };
}

impl std::error::Error for InstallerErr {}

impl fmt::Display for InstallerErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        forward_error_method!(self, fmt, f)
    }
}

impl MiruError for InstallerErr {
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
