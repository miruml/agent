// standard library
use std::fmt;

// internal crates
use crate::crypt::errors::CryptErr;
use crate::errors::{Code, HTTPCode, MiruError, Trace};

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
        write!(f, "Auth error: {}", self.source)
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
        write!(f, "Auth serialization error: {}", self.source)
    }
}


#[derive(Debug)]
pub enum AuthErr {
    CryptErr(AuthCryptErr),
    SerdeErr(SerdeErr),
}

macro_rules! forward_error_method {
    ($self:ident, $method:ident $(, $arg:expr)?) => {
        match $self {
            Self::CryptErr(e) => e.$method($($arg)?),
            Self::SerdeErr(e) => e.$method($($arg)?),
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