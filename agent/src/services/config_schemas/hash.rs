// internal crates
use crate::crypt::sha256;
use crate::http::prelude::*;
use crate::services::errors::{ServiceErr, ServiceHTTPErr, ServiceStorageErr};
use crate::storage::digests::{ConfigSchemaDigestCache, ConfigSchemaDigests};
use crate::trace;
use openapi_client::models::{
    HashSchemaSerializedRequest as ClientHashSchemaSerializedRequest,
    HashSerializedConfigSchemaFormat as ClientFormat,
};
use openapi_server::models::HashSerializedConfigSchemaFormat as ServerFormat;
use tracing::debug;

pub trait HashSchemaArgsI {
    fn schema(&self) -> &Vec<u8>;
    fn format(&self) -> &ServerFormat;
}

pub struct HashSchemaArgs {
    pub schema: Vec<u8>,
    pub format: ServerFormat,
}

impl HashSchemaArgsI for HashSchemaArgs {
    fn schema(&self) -> &Vec<u8> {
        &self.schema
    }

    fn format(&self) -> &ServerFormat {
        &self.format
    }
}

pub async fn hash_schema<ArgsT: HashSchemaArgsI, HTTPClientT: ConfigSchemasExt>(
    args: &ArgsT,
    cache: &ConfigSchemaDigestCache,
    http_client: &HTTPClientT,
    token: &str,
) -> Result<String, ServiceErr> {
    // raw digest of the schema (but we need the digest of the resolved schema)
    let raw_digest = sha256::hash_bytes(args.schema());
    debug!("Schema raw digest: {}", raw_digest);

    // check for the raw digest in the storage for the known schema digest
    let digests = cache.read_optional(raw_digest.clone()).await.map_err(|e| {
        ServiceErr::StorageErr(ServiceStorageErr {
            source: e,
            trace: trace!(),
        })
    })?;

    if let Some(digests) = digests {
        return Ok(digests.resolved);
    }

    // if not found, send the hash request to the server
    let hash_request = ClientHashSchemaSerializedRequest {
        schema: args.schema().to_vec(),
        format: server_format_to_client_format(args.format()),
    };
    let digest_response = http_client
        .hash_schema(&hash_request, token)
        .await
        .map_err(|e| {
            ServiceErr::HTTPErr(ServiceHTTPErr {
                source: e,
                trace: trace!(),
            })
        })?;

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
        .map_err(|e| {
            ServiceErr::StorageErr(ServiceStorageErr {
                source: e,
                trace: trace!(),
            })
        })?;

    // return the hash
    Ok(resolved_digest)
}

fn server_format_to_client_format(format: &ServerFormat) -> ClientFormat {
    match format {
        ServerFormat::HASH_SERIALIZED_CONFIG_SCHEMA_FORMAT_JSON => {
            ClientFormat::HASH_SERIALIZED_CONFIG_SCHEMA_FORMAT_JSON
        }
        ServerFormat::HASH_SERIALIZED_CONFIG_SCHEMA_FORMAT_YAML => {
            ClientFormat::HASH_SERIALIZED_CONFIG_SCHEMA_FORMAT_YAML
        }
    }
}
