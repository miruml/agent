// standard library
use std::fmt;

// internal crates
use crate::auth::errors::AuthErr;
use crate::crypt::errors::CryptErr;
use crate::errors::MiruError;
use crate::errors::{Code, HTTPCode, Trace};
use crate::filesys::errors::FileSysErr;
use crate::filesys::file::File;
use crate::http::errors::HTTPErr;
use crate::services::errors::ServiceErr;
use crate::storage::errors::StorageErr;

#[derive(Debug)]
pub struct MissingClientIDErr {
    pub agent_file_err: Box<FileSysErr>,
    pub jwt_err: Box<CryptErr>,
    pub trace: Box<Trace>,
}

impl MiruError for MissingClientIDErr {
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

impl fmt::Display for MissingClientIDErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unable to determine client id from the agent file or the token on file: agent file error: {}, jwt error: {}", self.agent_file_err, self.jwt_err)
    }
}

#[derive(Debug)]
pub struct ShutdownMngrDuplicateArgErr {
    pub arg_name: Box<String>,
    pub trace: Box<Trace>,
}

impl MiruError for ShutdownMngrDuplicateArgErr {
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

impl fmt::Display for ShutdownMngrDuplicateArgErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "shutdown manager was provided the same argument ({}) twice",
            self.arg_name
        )
    }
}

#[derive(Debug)]
pub struct ServerAuthErr {
    pub source: Box<AuthErr>,
    pub trace: Box<Trace>,
}

impl MiruError for ServerAuthErr {
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

impl fmt::Display for ServerAuthErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "server auth error: {}", self.source)
    }
}

#[derive(Debug)]
pub struct ServerCryptErr {
    pub source: Box<CryptErr>,
    pub trace: Box<Trace>,
}

impl MiruError for ServerCryptErr {
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

impl fmt::Display for ServerCryptErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "server crypt error: {}", self.source)
    }
}

#[derive(Debug)]
pub struct ServerFileSysErr {
    pub source: Box<FileSysErr>,
    pub trace: Box<Trace>,
}

impl MiruError for ServerFileSysErr {
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

impl fmt::Display for ServerFileSysErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "server file system error: {}", self.source)
    }
}

#[derive(Debug)]
pub struct ServerHTTPErr {
    pub source: Box<HTTPErr>,
    pub trace: Box<Trace>,
}

impl MiruError for ServerHTTPErr {
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

impl fmt::Display for ServerHTTPErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "http client error: {}", self.source)
    }
}

#[derive(Debug)]
pub struct ServerStorageErr {
    pub source: Box<StorageErr>,
    pub trace: Box<Trace>,
}

impl MiruError for ServerStorageErr {
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

impl fmt::Display for ServerStorageErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "server storage error: {}", self.source)
    }
}

#[derive(Debug)]
pub struct ServerServiceErr {
    pub source: Box<ServiceErr>,
    pub trace: Box<Trace>,
}

impl MiruError for ServerServiceErr {
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

impl fmt::Display for ServerServiceErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "server service error: {}", self.source)
    }
}

#[derive(Debug)]
pub struct BindUnixSocketErr {
    pub socket_file: File,
    pub source: Box<std::io::Error>,
    pub trace: Box<Trace>,
}

impl MiruError for BindUnixSocketErr {
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

impl fmt::Display for BindUnixSocketErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to bind unix socket '{}': {}",
            self.socket_file, self.source
        )
    }
}

#[derive(Debug)]
pub struct RunAxumServerErr {
    pub source: Box<std::io::Error>,
    pub trace: Box<Trace>,
}

impl MiruError for RunAxumServerErr {
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

impl fmt::Display for RunAxumServerErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to run axum server: {}", self.source)
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
    pub service: Box<String>,
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
    pub msg: Box<String>,
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
    MissingClientIDErr(MissingClientIDErr),
    TimestampConversionErr(TimestampConversionErr),
    ShutdownMngrDuplicateArgErr(ShutdownMngrDuplicateArgErr),

    // internal crate errors
    AuthErr(ServerAuthErr),
    CryptErr(ServerCryptErr),
    FileSysErr(ServerFileSysErr),
    HTTPErr(ServerHTTPErr),
    StorageErr(ServerStorageErr),
    ServiceErr(ServerServiceErr),

    // external crate errors
    BindUnixSocketErr(BindUnixSocketErr),
    RunAxumServerErr(RunAxumServerErr),
    SendShutdownSignalErr(SendShutdownSignalErr),
    JoinHandleErr(JoinHandleErr),
}

macro_rules! forward_error_method {
    ($self:ident, $method:ident $(, $arg:expr)?) => {
        match $self {
            Self::MissingClientIDErr(e) => e.$method($($arg)?),
            Self::TimestampConversionErr(e) => e.$method($($arg)?),
            Self::ShutdownMngrDuplicateArgErr(e) => e.$method($($arg)?),
            Self::AuthErr(e) => e.$method($($arg)?),
            Self::CryptErr(e) => e.$method($($arg)?),
            Self::FileSysErr(e) => e.$method($($arg)?),
            Self::HTTPErr(e) => e.$method($($arg)?),
            Self::StorageErr(e) => e.$method($($arg)?),
            Self::ServiceErr(e) => e.$method($($arg)?),
            Self::BindUnixSocketErr(e) => e.$method($($arg)?),
            Self::RunAxumServerErr(e) => e.$method($($arg)?),
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
