// internal crates
use crate::http::prelude::*;
use crate::services::errors::{
    ConfigSchemaNotFound, ServiceErr, ServiceHTTPErr, ServiceStorageErr, TooManyConfigSchemas,
};
use crate::storage::config_instances::ConfigInstanceCache;
use crate::storage::config_schemas::{
    ConfigSchemaCache, config_type_slug_and_schema_digest_filter,
};
use crate::trace;
use openapi_server::models::BaseConfigInstance;

// external crates
use serde::Deserialize;

pub trait ReadDeployedArgsI {
    fn device_id(&self) -> &str;
    fn config_type_slug(&self) -> &str;
    fn config_schema_digest(&self) -> &str;
}

#[derive(Debug, Deserialize)]
pub struct ReadDeployedArgs {
    pub device_id: String,
    pub config_type_slug: String,
    pub config_schema_digest: String,
}

impl ReadDeployedArgsI for ReadDeployedArgs {
    fn device_id(&self) -> &str {
        &self.device_id
    }
    fn config_type_slug(&self) -> &str {
        &self.config_type_slug
    }
    fn config_schema_digest(&self) -> &str {
        &self.config_schema_digest
    }
}

pub async fn read_deployed<ArgsT: ReadDeployedArgsI, HTTPClientT: ConfigSchemasExt>(
    args: &ArgsT,
    cache: &ConfigInstanceCache,
    http_client: &HTTPClientT,
    token: &str,
) -> Result<BaseConfigInstance, ServiceErr> {

    // thread 1: read the config schema id

    // thread 2: IF mqtt is disabled
    //     use the controller to refresh the agent's cache 

    // read the config instance from the cache

    Ok(BaseConfigInstance::default())
}

async fn fetch_config_schema_id<HTTPClientT: ConfigSchemasExt>(
    args: &ReadDeployedArgs,
    http_client: &HTTPClientT,
    cache: &ConfigSchemaCache,
    token: &str,
) -> Result<String, ServiceErr> {

    // search the cache for the config schema
    let digest = args.config_schema_digest().to_string();
    let config_type_slug = args.config_type_slug().to_string();
    let cfg_schema_entry = cache.find_one_entry_optional(move |entry| {
        config_type_slug_and_schema_digest_filter(entry, &config_type_slug, &digest)
    }).await.map_err(|e| {
        ServiceErr::StorageErr(ServiceStorageErr {
            source: e,
            trace: trace!(),
        })
    })?;
    if let Some(cfg_schema_entry) = cfg_schema_entry {
        return Ok(cfg_schema_entry.value.id.clone());
    }

    // search the backend for the config schema
    let cfg_schema = http_client.find_one_config_schema(
        [args.config_schema_digest()],
        [args.config_type_slug()],
        token,
    ).await.map_err(|e| {
        ServiceErr::HTTPErr(ServiceHTTPErr {
            source: e,
            trace: trace!(),
        })
    })?;

    Ok(cfg_schema.id)
}