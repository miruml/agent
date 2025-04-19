// internal crates
use crate::errors::MiruError;
use crate::errors::Trace;
use crate::http_client::errors::HTTPErr;
use crate::storage::errors::StorageErr;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum ServiceErr {
    #[error("Storage Error: {source}")]
    StorageErr {
        source: StorageErr,
        trace: Box<Trace>,
    },
    #[error("HTTP Error: {source}")]
    HTTPErr { source: HTTPErr, trace: Box<Trace> },
    #[error("Latest Concrete Config Not Found: {config_slug} {config_schema_digest}")]
    LatestConcreteConfigNotFound {
        config_slug: String,
        config_schema_digest: String,
        trace: Box<Trace>,
    },
}

impl AsRef<dyn MiruError> for ServiceErr {
    fn as_ref(&self) -> &(dyn MiruError + 'static) {
        self
    }
}

impl MiruError for ServiceErr {
    fn is_network_connection_error(&self) -> bool {
        matches!(
            self,
            ServiceErr::HTTPErr {
                source: HTTPErr::ConnectionErr { .. },
                ..
            }
        )
    }
}
