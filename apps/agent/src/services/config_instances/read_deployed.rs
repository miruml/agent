// standard crates
use std::sync::Arc;

// internal crates
use crate::crud::config_instance::matches_config_schema_and_activity_status;
use crate::crud::config_schema::matches_config_type_slug_and_schema_digest;
use crate::crud::prelude::*;
use crate::errors::MiruError;
use crate::http::prelude::*;
use crate::http::{
    config_schemas::{ConfigSchemaFilters, ConfigTypeSlugFilter, DigestFilter},
    search::SearchOperator,
};
use crate::models::config_instance::{ActivityStatus, ConfigInstance};
use crate::services::errors::{
    ConfigSchemaNotFound, DeployedConfigInstanceNotFound, ServiceCrudErr, ServiceErr,
    ServiceHTTPErr, ServiceSyncErr,
};
use crate::storage::config_instances::{ConfigInstanceCache, ConfigInstanceDataCache};
use crate::storage::config_schemas::ConfigSchemaCache;
use crate::sync::syncer::Syncer;
use crate::trace;
use openapi_server::models::BaseConfigInstance;

// external crates
use chrono::{TimeDelta, Utc};
use serde::Deserialize;
use tracing::error;

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

pub async fn read_deployed<ReadDeployedArgsT: ReadDeployedArgsI, HTTPClientT: ConfigSchemasExt>(
    args: &ReadDeployedArgsT,
    syncer: &Syncer,
    cfg_inst_cache: Arc<ConfigInstanceCache>,
    cfg_inst_data_cache: Arc<ConfigInstanceDataCache>,
    schema_cache: &ConfigSchemaCache,
    http_client: &HTTPClientT,
    token: &str,
) -> Result<BaseConfigInstance, ServiceErr> {
    let (config_schema_id_result, sync_result) = tokio::join!(
        fetch_config_schema_id(args, http_client, schema_cache, token),
        sync_with_backend(syncer, cfg_inst_cache.clone(), cfg_inst_data_cache.clone())
    );

    let config_schema_id = config_schema_id_result?;

    // ignore sync errors and only log them if they're not network connection errors
    if let Err(e) = sync_result {
        if !e.is_network_connection_error() {
            error!("error syncing config instances: {}", e);
        }
    }

    let entries = cfg_inst_cache.find_entries_where(|_| true).await.unwrap();
    println!("entries: {:?}", entries);

    // read the config instance metadata from the cache
    let config_schema_id_cloned = config_schema_id.clone();
    let result = cfg_inst_cache
        .find_one_optional(
            "filter by config schema and activity status",
            move |cfg_inst| {
                matches_config_schema_and_activity_status(
                    cfg_inst,
                    &config_schema_id_cloned,
                    ActivityStatus::Deployed,
                )
            },
        )
        .await;

    let result = match result {
        Ok(metadata) => metadata,
        Err(e) => {
            if e.is_network_connection_error() {
                return Err(ServiceErr::DeployedConfigInstanceNotFound(Box::new(
                    DeployedConfigInstanceNotFound {
                        config_schema_id: config_schema_id.clone(),
                        config_type_slug: args.config_type_slug().to_string(),
                        config_schema_digest: args.config_schema_digest().to_string(),
                        network_connection_error: true,
                        trace: trace!(),
                    },
                )));
            } else {
                return Err(ServiceErr::CrudErr(Box::new(ServiceCrudErr {
                    source: e,
                    trace: trace!(),
                })));
            }
        }
    };

    // if we can't find the *metadata*, the deployed config instance doesn't exist or
    // couldn't be retrieved from the server due to a network connection error
    let metadata = match result {
        Some(metadata) => metadata,
        None => {
            return Err(ServiceErr::DeployedConfigInstanceNotFound(Box::new(
                DeployedConfigInstanceNotFound {
                    config_schema_id: config_schema_id.clone(),
                    config_type_slug: args.config_type_slug().to_string(),
                    config_schema_digest: args.config_schema_digest().to_string(),
                    network_connection_error: false,
                    trace: trace!(),
                },
            )));
        }
    };

    // if we can't find the *data*, there was an internal error somewhere because if
    // the metadata exists, the data should exist too
    let data = cfg_inst_data_cache
        .read(metadata.id.clone())
        .await
        .map_err(|e| {
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
    let result = cache
        .find_one_optional(
            "filter by config type slug and schema digest",
            move |cfg_sch| {
                matches_config_type_slug_and_schema_digest(cfg_sch, &config_type_slug, &digest)
            },
        )
        .await
        .map_err(|e| {
            ServiceErr::CrudErr(Box::new(ServiceCrudErr {
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
    let result = http_client.find_one_config_schema(filters, token).await;

    match result {
        Ok(cfg_schema) => Ok(cfg_schema.id),
        Err(e) => {
            if e.is_network_connection_error() {
                Err(ServiceErr::ConfigSchemaNotFound(Box::new(
                    ConfigSchemaNotFound {
                        digest: args.config_schema_digest().to_string(),
                        config_type_slug: args.config_type_slug().to_string(),
                        network_connection_error: true,
                        trace: trace!(),
                    },
                )))
            } else {
                Err(ServiceErr::HTTPErr(Box::new(ServiceHTTPErr {
                    source: e,
                    trace: trace!(),
                })))
            }
        }
    }
}

async fn sync_with_backend(
    syncer: &Syncer,
    cfg_inst_cache: Arc<ConfigInstanceCache>,
    cfg_inst_data_cache: Arc<ConfigInstanceDataCache>,
) -> Result<(), ServiceErr> {
    let last_synced_at = syncer.get_last_synced_at().await.map_err(|e| {
        ServiceErr::SyncErr(Box::new(ServiceSyncErr {
            source: e,
            trace: trace!(),
        }))
    })?;

    // don't sync if the last sync was less than 5 seconds ago
    if last_synced_at > Utc::now() - TimeDelta::seconds(5) {
        return Ok(());
    }

    syncer
        .sync(cfg_inst_cache, cfg_inst_data_cache, true)
        .await
        .map_err(|e| {
            ServiceErr::SyncErr(Box::new(ServiceSyncErr {
                source: e,
                trace: trace!(),
            }))
        })
}
