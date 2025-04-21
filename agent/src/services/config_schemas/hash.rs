// internal crates
use crate::http::prelude::*;
use crate::services::errors::{
    ServiceErr,
    ServiceHTTPErr,
    ServiceStorageErr,
};
use crate::storage::digests::{ConfigSchemaDigestCache, ConfigSchemaDigests};
use crate::trace;
use crate::utils;
use openapi_client::models::HashSchemaRequest;
use tracing::debug;

// external crates
use serde_json::Value;

pub trait HashSchemaArgsI {
    fn schema(&self) -> &Value;
}

pub struct HashSchemaArgs {
    pub schema: Value,
}

impl HashSchemaArgsI for HashSchemaArgs {
    fn schema(&self) -> &Value {
        &self.schema
    }
}

pub async fn hash_schema<ArgsT: HashSchemaArgsI, HTTPClientT: ConfigSchemasExt>(
    args: &ArgsT,
    http_client: &HTTPClientT,
    cache: &ConfigSchemaDigestCache,
) -> Result<String, ServiceErr> {
    // raw digest of the schema (but we need the digest of the resolved schema)
    let raw_digest = utils::hash_json(args.schema());
    debug!("Schema raw digest: {}", raw_digest);

    // check for the raw digest in the storage for the known schema digest
    let digests =
        cache
            .read_optional(raw_digest.clone())
            .await
            .map_err(|e| ServiceErr::StorageErr(ServiceStorageErr {
                source: e,
                trace: trace!(),
            }))?;

    if let Some(digests) = digests {
        return Ok(digests.resolved);
    }

    // if not found, send the hash request to the server
    let hash_request = HashSchemaRequest {
        schema: args.schema().clone(),
    };
    let digest_response =
        http_client
            .hash_schema(&hash_request)
            .await
            .map_err(|e| ServiceErr::HTTPErr(ServiceHTTPErr {
                source: e,
                trace: trace!(),
            }))?;

    // save the hash to the storage module
    let resolved_digest = digest_response.digest;
    let digests = ConfigSchemaDigests {
        raw: raw_digest.clone(),
        resolved: resolved_digest.clone(),
    };
    cache
        .write(
            raw_digest, digests,
            // this overwrite shouldn't ever occur since we check the storage first but no
            // reason to throw an error
            true,
        )
        .await
        .map_err(|e| ServiceErr::StorageErr(ServiceStorageErr {
            source: e,
            trace: trace!(),
        }))?;

    // return the hash
    Ok(resolved_digest)
}
