// internal crates
use crate::http_client::prelude::*;
use crate::errors::MiruError;
use crate::services::errors::ServiceErr;
use crate::storage::concrete_configs::ConcreteConfigCache;
use crate::trace;
use openapi_client::models::BackendConcreteConfig;

pub async fn read_latest<T: ConcreteConfigsExt>(
    config_slug: &str,
    config_schema_digest: &str,
    http_client: &T,
    cache: &ConcreteConfigCache,
) -> Result<BackendConcreteConfig, ServiceErr> {

    // read the latest concrete config from the server
    let result = http_client.read_latest_concrete_config(
        config_slug,
        config_schema_digest,
    ).await;

    // if not a network connection error, return the error (ignore network connection
    // errors)
    let result = match result {
        Ok(result) => result,
        Err(e) => {
            if !e.is_network_connection_error() {
                return Err(ServiceErr::HTTPErr {
                    source: e,
                    trace: trace!(),
                });
            }
            // if is a network connection error, then just return None and continue
            None
        }
    };

    // if successful, update the concrete config in storage and return it
    if let Some(concrete_config) = result {
        cache.write(
            config_slug,
            config_schema_digest,
            concrete_config.clone(),
            true,
        ).await.map_err(|e| ServiceErr::StorageErr {
            source: e,
            trace: trace!(),
        })?;
        return Ok(concrete_config);
    }

    // if unsuccessful, attempt to read the latest concrete config from storage
    let latest_concrete_config = cache.read_optional(
        config_slug,
        config_schema_digest,
    ).await.map_err(|e| ServiceErr::StorageErr {
        source: e,
        trace: trace!(),
    })?;

    match latest_concrete_config {
        Some(latest_concrete_config) => {
            Ok(latest_concrete_config)
        }
        None => {
            Err(ServiceErr::LatestConcreteConfigNotFound {
                config_slug: config_slug.to_string(),
                config_schema_digest: config_schema_digest.to_string(),
                trace: trace!(),
            })
        }
    }
}