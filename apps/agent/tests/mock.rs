// standard library
use std::fmt;

// internal crates
use config_agent::errors::{Code, HTTPCode, MiruError};

#[derive(Debug)]
pub struct MockMiruError {
    network_err: bool,
}

impl MockMiruError {
    pub fn new(network_err: bool) -> Self {
        Self { network_err }
    }
}

impl MiruError for MockMiruError {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        self.network_err
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for MockMiruError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MockMiruError")
    }
}
