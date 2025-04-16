// internal crates
use crate::http_client::client::HTTPClient;
use crate::errors::MiruError;
use crate::services::errors::ServiceErr;
use crate::storage::StorageLayout;
use crate::trace;
use openapi_client::models::BackendConcreteConfig;

pub async fn read_latest(
    config_slug: &str,
    config_schema_digest: &str,
) -> Result<BackendConcreteConfig, ServiceErr> {

    // read the latest concrete config from the server
    let http_client = HTTPClient::new().await;
    let result = http_client.read_latest_concrete_config(
        config_slug,
        config_schema_digest,
    ).await;

    // if not a network connection error, return the error (ignore network connection
    // errors)
    if let Err(e) = result {
        if !e.is_poor_signal_error() {
            return Err(ServiceErr::HTTPErr {
                source: e,
                trace: trace!(),
            });
        }
    }

    let storage_layout = StorageLayout::new_default()
        .map_err(|e| ServiceErr::StorageErr {
            source: e,
            trace: trace!(),
        })?;
    
    return Err(ServiceErr::HTTPErr {
        source: result,
        trace: trace!(),
    });
}


// if success
//      create or update the concrete config in the storage module 
//      update the concrete config cache information in the storage module
//      return the concrete config

// use the digest + config slug to get the cached schema id

// if not found, return not found error

// use the cached schema id to get the latest concrete config

// if not found, return not found error

// return the latest concrete config