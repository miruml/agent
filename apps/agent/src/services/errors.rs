// standard library
use std::fmt;

// internal crates
use crate::errors::{Code, HTTPCode, MiruError, Trace};
use crate::http::errors::HTTPErr;
use crate::models::errors::ModelsErr;
use crate::storage::errors::StorageErr;

// external crates
use serde_json::json;

#[derive(Debug)]
pub struct ConfigSchemaNotFound {
    pub config_type_slug: String,
    pub config_schema_digest: String,
    pub trace: Box<Trace>,
}

impl MiruError for ConfigSchemaNotFound {
    fn code(&self) -> Code {
        Code::ResourceNotFound
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::NOT_FOUND
    }

    fn is_network_connection_error(&self) -> bool {
        false
    }

    fn params(&self) -> Option<serde_json::Value> {
        Some(json!({
            "config_type_slug": self.config_type_slug,
            "config_schema_digest": self.config_schema_digest,
        }))
    }
}

impl fmt::Display for ConfigSchemaNotFound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Config schema with config type slug '{}' and digest '{}' not found", self.config_type_slug, self.config_schema_digest)
    }
}

#[derive(Debug)]
pub struct TooManyConfigSchemas {
    pub config_schema_ids: Vec<String>,
    pub config_type_slug: String,
    pub config_schema_digest: String,
    pub trace: Box<Trace>,
}

impl MiruError for TooManyConfigSchemas {
    fn code(&self) -> Code {
        Code::ResourceNotFound
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::NOT_FOUND
    }

    fn is_network_connection_error(&self) -> bool {
        false
    }

    fn params(&self) -> Option<serde_json::Value> {
        Some(json!({
            "config_schema_ids": self.config_schema_ids,
            "config_type_slug": self.config_type_slug,
            "config_schema_digest": self.config_schema_digest,
        }))
    }
}

impl fmt::Display for TooManyConfigSchemas {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Found more than one config schema (ids: {}) with config type slug '{}' and digest '{}'", self.config_schema_ids.join(", "), self.config_type_slug, self.config_schema_digest)
    }
}


#[derive(Debug)]
pub struct LatestConfigInstanceNotFound {
    pub config_type_slug: String,
    pub config_schema_digest: String,
    pub trace: Box<Trace>,
}

impl MiruError for LatestConfigInstanceNotFound {
    fn code(&self) -> Code {
        Code::ResourceNotFound
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::NOT_FOUND
    }

    fn is_network_connection_error(&self) -> bool {
        false
    }

    fn params(&self) -> Option<serde_json::Value> {
        Some(json!({
            "config_type_slug": self.config_type_slug,
            "config_schema_digest": self.config_schema_digest,
        }))
    }
}

impl fmt::Display for LatestConfigInstanceNotFound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unable to locate the latest config instance for config type slug: '{}' and config schema digest: '{}'", self.config_type_slug, self.config_schema_digest)
    }
}

#[derive(Debug)]
pub struct ServiceModelsErr {
    pub source: ModelsErr,
    pub trace: Box<Trace>,
}

impl MiruError for ServiceModelsErr {
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

impl fmt::Display for ServiceModelsErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Models Error: {}", self.source)
    }
}

#[derive(Debug)]
pub struct ServiceStorageErr {
    pub source: StorageErr,
    pub trace: Box<Trace>,
}

impl MiruError for ServiceStorageErr {
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

impl fmt::Display for ServiceStorageErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Storage Error: {}", self.source)
    }
}

#[derive(Debug)]
pub struct ServiceHTTPErr {
    pub source: HTTPErr,
    pub trace: Box<Trace>,
}

impl MiruError for ServiceHTTPErr {
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

impl fmt::Display for ServiceHTTPErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "service HTTP Error: {}", self.source)
    }
}

#[derive(Debug)]
pub enum ServiceErr {
    // service errors
    ConfigSchemaNotFound(ConfigSchemaNotFound),
    TooManyConfigSchemas(TooManyConfigSchemas),
    LatestConfigInstanceNotFound(LatestConfigInstanceNotFound),

    // internal crate errors
    ModelsErr(ServiceModelsErr),
    StorageErr(ServiceStorageErr),
    HTTPErr(ServiceHTTPErr),
}

macro_rules! forward_error_method {
    ($self:ident, $method:ident $(, $arg:expr)?) => {
        match $self {
            Self::ConfigSchemaNotFound(e) => e.$method($($arg)?),
            Self::TooManyConfigSchemas(e) => e.$method($($arg)?),
            Self::LatestConfigInstanceNotFound(e) => e.$method($($arg)?),
            Self::ModelsErr(e) => e.$method($($arg)?),
            Self::StorageErr(e) => e.$method($($arg)?),
            Self::HTTPErr(e) => e.$method($($arg)?),
        }
    };
}

impl fmt::Display for ServiceErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        forward_error_method!(self, fmt, f)
    }
}

impl MiruError for ServiceErr {
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
