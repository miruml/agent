// internal crates
use crate::crud::prelude::*;
use crate::crud::config_instance::matches_config_schema_and_activity_status;
use crate::crud::config_schema::matches_config_type_slug_and_schema_digest;
use crate::http::prelude::*;
use crate::http::{
    config_schemas::{ConfigSchemaFilters, DigestFilter, ConfigTypeSlugFilter},
    search::SearchOperator,
};
use crate::models::config_instance::{
    ActivityStatus,
    ConfigInstance,
};
use crate::services::errors::{
    ServiceErr, ServiceHTTPErr, ServiceCrudErr, DeployedConfigInstanceNotFound
};
use crate::storage::config_instances::{ConfigInstanceCache, ConfigInstanceDataCache};
use crate::storage::config_schemas::ConfigSchemaCache;
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

pub async fn read_deployed<
    ReadDeployedArgsT: ReadDeployedArgsI,
    HTTPClientT: ConfigSchemasExt,
>(
    args: &ReadDeployedArgsT,
    instance_cache: &ConfigInstanceCache,
    instance_data_cache: &ConfigInstanceDataCache,
    schema_cache: &ConfigSchemaCache,
    http_client: &HTTPClientT,
    token: &str,
) -> Result<BaseConfigInstance, ServiceErr> {

    // thread 1: 
    let config_schema_id = fetch_config_schema_id(
        args, http_client, schema_cache, token
    ).await?;

    // thread 2: 
    // if !mqtt_enabled {
        //     use the controller to refresh the agent's cache 
        // TODO: implement this
    // }

    // read the config instance metadata from the cache
    let result = instance_cache.find_one_optional(
        "filter by config schema and activity status",
        move |cfg_inst| {
            matches_config_schema_and_activity_status(cfg_inst, &config_schema_id, ActivityStatus::Deployed)
        }
    ).await.map_err(|e| {
        ServiceErr::CrudErr(Box::new(ServiceCrudErr {
            source: e,
            trace: trace!(),
        }))
    })?;

    // if we can't find the *metadata*, the deployed config instance doesn't exist or
    // couldn't be retrieved from the server due to a network connection error
    let metadata = match result {
        Some(metadata) => metadata,
        None => {
            return Err(ServiceErr::DeployedConfigInstanceNotFound(Box::new(DeployedConfigInstanceNotFound {
                config_type_slug: args.config_type_slug().to_string(),
                config_schema_digest: args.config_schema_digest().to_string(),
                trace: trace!(),
            })));
        }
    };

    // if we can't find the *data*, there was an internal error somewhere because if
    // the metadata exists, the data should exist too
    let data = instance_data_cache.read(metadata.id.clone()).await.map_err(|e| {
        ServiceErr::CrudErr(Box::new(ServiceCrudErr {
            source: e,
            trace: trace!(),
        }))
    })?;

    Ok(ConfigInstance::to_sdk(metadata, data))
}

async fn fetch_config_schema_id<
    ReadDeployedArgsT: ReadDeployedArgsI,
    HTTPClientT: ConfigSchemasExt,
>(
    args: &ReadDeployedArgsT,
    http_client: &HTTPClientT,
    cache: &ConfigSchemaCache,
    token: &str,
) -> Result<String, ServiceErr> {

    // search the cache for the config schema
    let digest = args.config_schema_digest().to_string();
    let config_type_slug = args.config_type_slug().to_string();
    let result = cache.find_one_optional(
        "filter by config type slug and schema digest",
        move |cfg_sch| {
            matches_config_type_slug_and_schema_digest(cfg_sch, &config_type_slug, &digest)
        }
    ).await.map_err(|e| {
        ServiceErr::CrudErr(Box::new(ServiceCrudErr{
            source: e,
            trace: trace!(),
        }))
    })?;
    if let Some(cfg_schema) = result {
        return Ok(cfg_schema.id.clone());
    }

    // search the backend for the config schema
    let filters = ConfigSchemaFilters {
        digests: Some(DigestFilter {
            not: false,
            op: SearchOperator::Equals,
            val: vec![args.config_schema_digest().to_string()],
        }),
        config_type_slugs: Some(ConfigTypeSlugFilter {
            not: false,
            op: SearchOperator::Equals,
            val: vec![args.config_type_slug().to_string()],
        }),
    };
    let cfg_schema = http_client.find_one_config_schema(
        filters,
        token,
    ).await.map_err(|e| {
        ServiceErr::HTTPErr(Box::new(ServiceHTTPErr {
            source: e,
            trace: trace!(),
        }))
    })?;

    Ok(cfg_schema.id)
}