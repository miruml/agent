// internal crates
use crate::http::prelude::*;
use crate::services::errors::{
    ConfigSchemaNotFound, ServiceErr, ServiceHTTPErr, TooManyConfigSchemas,
};
use crate::storage::config_instances::ConfigInstanceCache;
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

async fn get_config_schema_id<HTTPClientT: ConfigSchemasExt>(
    args: &ReadDeployedArgs,
    http_client: &HTTPClientT,
    // add a cache here
    token: &str,
) -> Result<String, ServiceErr> {

    // TODO: implement and check the cache for the config schema using the digest and
    // config type slug. Do this using a cache implemented index

    // read the config schemas by digest and config type
    let cfg_schemas = http_client.list_config_schemas(
        &[args.config_schema_digest().to_string()],
        &[args.config_type_slug().to_string()],
        token,
    ).await.map_err(|e| {
        ServiceErr::HTTPErr(ServiceHTTPErr {
            source: e,
            trace: trace!(),
        })
    })?;

    // read the first config schema from the list
    let cfg_schema = match cfg_schemas.data.first() {
        Some(cfg_schema) => cfg_schema,
        None => {
            return Err(ServiceErr::ConfigSchemaNotFound(ConfigSchemaNotFound {
                config_type_slug: args.config_type_slug().to_string(),
                config_schema_digest: args.config_schema_digest().to_string(),
                trace: trace!(),
            }));
        }
    };

    // check that there is only one config schema
    if cfg_schemas.data.len() > 1 {
        return Err(ServiceErr::TooManyConfigSchemas(TooManyConfigSchemas {
            config_schema_ids: cfg_schemas.data.iter().map(|c| c.id.clone()).collect(),
            config_type_slug: args.config_type_slug().to_string(),
            config_schema_digest: args.config_schema_digest().to_string(),
            trace: trace!(),
        }));
    }

    Ok(cfg_schema.id.clone())
}