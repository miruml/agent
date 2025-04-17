// internal crates
use crate::http_client::prelude::*;
use crate::services::errors::ServiceErr;
use crate::storage::digests::{
    AsyncConfigSchemaDigestCache,
    ConfigSchemaDigests,
};
use crate::trace;
use crate::utils;
use openapi_client::models::HashSchemaRequest;

// external crates
use serde_json::Value;

pub async fn hash_schema<T: ConfigSchemasExt>(
    schema: &Value,
    http_client: &T,
    cache: &AsyncConfigSchemaDigestCache,
) -> Result<String, ServiceErr> {

    // raw digest of the schema (but we need the digest of the resolved schema)
    let raw_digest = utils::hash_json(schema);

    // check for the raw digest in the storage for the known schema digest
    let digests= cache.read_optional(&raw_digest).await
        .map_err(|e| ServiceErr::StorageErr {
            source: e,
            trace: trace!(),
        })?;

    if let Some(digests) = digests {
        return Ok(digests.resolved);
    }

    // if not found, send the hash request to the server
    let hash_request = HashSchemaRequest { schema: schema.clone() };
    let digest_response = http_client.hash_schema(&hash_request).await
        .map_err(|e| ServiceErr::HTTPErr {
            source: e,
            trace: trace!(),
        })?;

    // save the hash to the storage module
    let resolved_digest = digest_response.digest;
    let digests = ConfigSchemaDigests {
        raw: raw_digest,
        resolved: resolved_digest.clone(),
    };
    cache.write(
        digests,
        true,
    ).await
    .map_err(|e| ServiceErr::StorageErr {
        source: e,
        trace: trace!(),
    })?;

    // return the hash
    Ok(resolved_digest)
}

