// internal crates
use crate::http_client::client::HTTPClient;
use crate::services::errors::ServiceErr;
use crate::storage::layout::StorageLayout;
use crate::trace;
use crate::utils;
use openapi_client::models::HashSchemaRequest;

// external crates
use serde_json::Value;

pub async fn hash_schema(schema: &Value) -> Result<String, ServiceErr> {

    // raw digest of the schema (but we need the digest of the resolved schema)
    let raw_digest = utils::hash_json(schema);

    // check for the raw digest in the storage for the known schema digest
    let storage_layout = StorageLayout::new_default();

    let cfg_sch_digest_registry = storage_layout.cfg_sch_digest_registry();
    let resolved_digest = cfg_sch_digest_registry.read_resolved_digest(&raw_digest)
        .map_err(|e| ServiceErr::StorageErr {
            source: e,
            trace: trace!(),
        })?;

    if let Some(resolved_digest) = resolved_digest {
        return Ok(resolved_digest);
    }

    // if not found, send the hash request to the server
    let http_client = HTTPClient::new().await;
    let hash_request = HashSchemaRequest { schema: schema.clone() };
    let digest_response = http_client.hash_schema(&hash_request).await
        .map_err(|e| ServiceErr::HTTPErr {
            source: e,
            trace: trace!(),
        })?;

    // FIXME: use atomic writes to the registry

    // save the hash to the storage module
    cfg_sch_digest_registry.insert(
        &raw_digest,
        &digest_response.digest,
        true,
    )
    .map_err(|e| ServiceErr::StorageErr {
        source: e,
        trace: trace!(),
    })?;

    // return the hash
    Ok(digest_response.digest)
}

