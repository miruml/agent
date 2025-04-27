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
pub struct UknownOSUserErr {
    pub trace: Box<Trace>,
}

impl MiruError for UknownOSUserErr {
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

impl fmt::Display for UknownOSUserErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Unable to determine the os user (must be run as the 'miru' user)"
        )
    }
}

#[derive(Debug)]
pub struct UknownOSGroupErr {
    pub trace: Box<Trace>,
}

impl MiruError for UknownOSGroupErr {
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

impl fmt::Display for UknownOSGroupErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Unable to determine the os group (must be run as the 'miru' group)"
        )
    }
}

#[derive(Debug)]
pub struct InvalidOSUserErr {
    pub target_user: String,
    pub actual_user: String,
    pub trace: Box<Trace>,
}

impl MiruError for InvalidOSUserErr {
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

impl fmt::Display for InvalidOSUserErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Installation must be run as the '{}' user (prepend with sudo -u '{}') not the '{}' user", self.target_user, self.target_user, self.actual_user)
    }
}

#[derive(Debug)]
pub struct InvalidOSGroupErr {
    pub target_group: String,
    pub actual_group: String,
    pub trace: Box<Trace>,
}

impl MiruError for InvalidOSGroupErr {
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

impl fmt::Display for InvalidOSGroupErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Installation must be run as the '{}' group (prepend with sudo -g '{}') not the '{}' group", self.target_group, self.target_group, self.actual_group)
    }
}

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
    // installer errors
    UknownOSUserErr(UknownOSUserErr),
    UknownOSGroupErr(UknownOSGroupErr),
    InvalidOSUserErr(InvalidOSUserErr),
    InvalidOSGroupErr(InvalidOSGroupErr),

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
            // installer errors
            Self::UknownOSUserErr(e) => e.$method($($arg)?),
            Self::UknownOSGroupErr(e) => e.$method($($arg)?),
            Self::InvalidOSUserErr(e) => e.$method($($arg)?),
            Self::InvalidOSGroupErr(e) => e.$method($($arg)?),

            // internal crate errors
            Self::AuthErr(e) => e.$method($($arg)?),
            Self::CryptErr(e) => e.$method($($arg)?),
            Self::FileSysErr(e) => e.$method($($arg)?),
            Self::HTTPErr(e) => e.$method($($arg)?),
            Self::StorageErr(e) => e.$method($($arg)?),

            // external crate errors
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
