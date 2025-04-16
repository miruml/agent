// internal crates
use crate::storage::errors::StorageErr;
use crate::errors::Trace;
use crate::http_client::errors::HTTPErr;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum ServiceErr {
    #[error("Storage Error: {source}")]
    StorageErr { source: StorageErr, trace: Box<Trace> },
    #[error("HTTP Error: {source}")]
    HTTPErr { source: HTTPErr, trace: Box<Trace> },
}
