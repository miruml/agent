use crate::crud::prelude::*;
use crate::http::{
    config_instances::{
        ActivityStatusFilter, ConfigInstanceFiltersBuilder, ConfigInstancesExt, IDFilter,
    },
    search::SearchOperator,
};
use crate::models::config_instance::{ConfigInstance, TargetStatus};
use crate::storage::config_instances::{ConfigInstanceCache, ConfigInstanceContentCache};
use crate::sync::errors::*;
use crate::trace;
use openapi_client::models::{
    ConfigInstance as BackendConfigInstance, ConfigInstanceActivityStatus, ConfigInstanceExpand,
};

// external crates
use tracing::{debug, error};

pub async fn pull_config_instances<HTTPClientT: ConfigInstancesExt>(
    cfg_inst_cache: &ConfigInstanceCache,
    cfg_inst_content_cache: &ConfigInstanceContentCache,
    http_client: &HTTPClientT,
    device_id: &str,
    token: &str,
) -> Result<(), SyncErr> {
    let unremoved_insts = fetch_unremoved_instances(http_client, device_id, token).await?;
    debug!(
        "Found {} unremoved config instances: {:?}",
        unremoved_insts.len(),
        unremoved_insts
    );

    let categorized_insts = categorize_instances(cfg_inst_cache, unremoved_insts).await?;
    debug!(
        "Found {} unknown config instances: {:?}",
        categorized_insts.unknown.len(),
        categorized_insts.unknown
    );
    debug!(
        "Found {} instances with updated target status: {:?}",
        categorized_insts.update_target_status.len(),
        categorized_insts.update_target_status
    );

    let unknown_insts = fetch_instances_with_expanded_instance_data(
        http_client,
        device_id,
        categorized_insts
            .unknown
            .iter()
            .map(|inst| inst.id.clone())
            .collect(),
        token,
    )
    .await?;

    debug!(
        "Adding {} unknown instances to storage",
        unknown_insts.len()
    );
    add_unknown_instances_to_storage(cfg_inst_cache, cfg_inst_content_cache, unknown_insts).await?;

    debug!(
        "Updating target status for {} instances",
        categorized_insts.update_target_status.len()
    );
    update_target_status_instances(cfg_inst_cache, categorized_insts.update_target_status).await?;

    Ok(())
}

async fn fetch_unremoved_instances<HTTPClientT: ConfigInstancesExt>(
    http_client: &HTTPClientT,
    device_id: &str,
    token: &str,
) -> Result<Vec<BackendConfigInstance>, SyncErr> {
    let filters = ConfigInstanceFiltersBuilder::new(device_id.to_string())
        .with_activity_status_filter(ActivityStatusFilter {
            not: false,
            op: SearchOperator::Equals,
            val: vec![
                ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_QUEUED,
                ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_DEPLOYED,
            ],
        })
        .build();
    http_client
        .list_all_config_instances(filters, &[] as &[ConfigInstanceExpand], token)
        .await
        .map_err(|e| {
            SyncErr::HTTPClientErr(Box::new(SyncHTTPClientErr {
                source: e,
                trace: trace!(),
            }))
        })
}

#[derive(Debug)]
pub struct CategorizedConfigInstances {
    pub unknown: Vec<BackendConfigInstance>,
    pub update_target_status: Vec<ConfigInstance>,
    pub other: Vec<BackendConfigInstance>,
}

async fn categorize_instances(
    cfg_inst_cache: &ConfigInstanceCache,
    unremoved_insts: Vec<BackendConfigInstance>,
) -> Result<CategorizedConfigInstances, SyncErr> {
    let mut categorized = CategorizedConfigInstances {
        unknown: Vec::new(),
        update_target_status: Vec::new(),
        other: Vec::new(),
    };

    // deleteme
    match cfg_inst_cache.entries().await {
        Ok(entries) => {
            debug!("Found {} instances in cache: {:?}", entries.len(), entries);
        }
        Err(e) => {
            error!("Failed to read config instances from cache: {}", e);
        }
    }

    // unknown config instances
    for server_inst in unremoved_insts {
        // check if the config instance is known
        let mut storage_inst = match cfg_inst_cache
            .read_optional(server_inst.id.clone())
            .await
            .map_err(|e| {
                SyncErr::CrudErr(Box::new(SyncCrudErr {
                    source: e,
                    trace: trace!(),
                }))
            })? {
            Some(storage_inst) => {
                debug!("Found config instance {}in cache", storage_inst.id);
                storage_inst
            }
            None => {
                debug!("Config instance {} not found in cache", server_inst.id);
                categorized.unknown.push(server_inst);
                continue;
            }
        };

        // check if the target status matches
        if storage_inst.target_status != TargetStatus::from_backend(&server_inst.target_status) {
            debug!(
                "Config instance {} has updated target status",
                storage_inst.id
            );
            storage_inst.target_status = TargetStatus::from_backend(&server_inst.target_status);
            categorized.update_target_status.push(storage_inst);
        } else {
            debug!(
                "Config instance {} has the same target status",
                storage_inst.id
            );
            categorized.other.push(server_inst);
        }
    }

    Ok(categorized)
}

async fn fetch_instances_with_expanded_instance_data<HTTPClientT: ConfigInstancesExt>(
    http_client: &HTTPClientT,
    device_id: &str,
    ids: Vec<String>,
    token: &str,
) -> Result<Vec<BackendConfigInstance>, SyncErr> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    // read the unknown config instances from the server with config instance content expanded
    let filters = ConfigInstanceFiltersBuilder::new(device_id.to_string())
        .with_id_filter(IDFilter {
            not: false,
            op: SearchOperator::Equals,
            val: ids.clone(),
        })
        .build();
    let cfg_insts = http_client
        .list_all_config_instances(
            filters,
            [ConfigInstanceExpand::CONFIG_INSTANCE_EXPAND_CONTENT],
            token,
        )
        .await
        .map_err(|e| {
            SyncErr::HTTPClientErr(Box::new(SyncHTTPClientErr {
                source: e,
                trace: trace!(),
            }))
        })?;

    if cfg_insts.len() != ids.len() {
        return Err(SyncErr::MissingExpandedInstancesErr(Box::new(
            MissingExpandedInstancesErr {
                expected_ids: ids,
                actual_ids: cfg_insts.iter().map(|inst| inst.id.clone()).collect(),
                trace: trace!(),
            },
        )));
    }

    Ok(cfg_insts)
}

async fn add_unknown_instances_to_storage(
    cfg_inst_cache: &ConfigInstanceCache,
    cfg_inst_content_cache: &ConfigInstanceContentCache,
    unknown_insts: Vec<BackendConfigInstance>,
) -> Result<(), SyncErr> {
    // add the unknown config instances to the cache
    for mut unknown_inst in unknown_insts {
        // throw an error since if the config instance content isn't expanded for this
        // one it won't be expanded for any others and none of the config instances will
        // therefore be added to the cache
        let cfg_inst_content = match unknown_inst.content {
            Some(cfg_inst_content) => cfg_inst_content,
            None => {
                return Err(SyncErr::ConfigInstanceContentNotFound(Box::new(
                    ConfigInstanceContentNotFoundErr {
                        cfg_inst_id: unknown_inst.id.clone(),
                        trace: trace!(),
                    },
                )));
            }
        };
        unknown_inst.content = None;

        let overwrite = true;
        if let Err(e) = cfg_inst_content_cache
            .write(
                unknown_inst.id.clone(),
                cfg_inst_content,
                |_, _| false,
                overwrite,
            )
            .await
        {
            error!(
                "Failed to write config instance '{}' content to cache: {}",
                unknown_inst.id, e
            );
            continue;
        }

        let unknown_inst_id = unknown_inst.id.clone();
        let storage_inst = ConfigInstance::from_backend(unknown_inst);
        let overwrite = true;
        if let Err(e) = cfg_inst_cache
            .write(
                unknown_inst_id.clone(),
                storage_inst,
                |_, _| false,
                overwrite,
            )
            .await
        {
            error!(
                "Failed to write config instance '{}' to cache: {}",
                unknown_inst_id, e
            );
            continue;
        }
    }
    Ok(())
}

async fn update_target_status_instances(
    cfg_inst_cache: &ConfigInstanceCache,
    update_target_status: Vec<ConfigInstance>,
) -> Result<(), SyncErr> {
    for cfg_inst in update_target_status {
        let cfg_inst_id = cfg_inst.id.clone();

        // read the config instance from the cache to update only select fields
        let cache_inst = match cfg_inst_cache.read(cfg_inst_id.clone()).await {
            Ok(cache_inst) => cache_inst,
            Err(e) => {
                error!(
                    "Failed to read config instance '{}' from cache: {}",
                    cfg_inst_id, e
                );
                continue;
            }
        };
        let updated_inst = ConfigInstance {
            target_status: cfg_inst.target_status,
            updated_by_id: cfg_inst.updated_by_id,
            updated_at: cfg_inst.updated_at,
            ..cache_inst
        };

        // write the updated config instance to the cache
        let overwrite = true;
        if let Err(e) = cfg_inst_cache
            .write(cfg_inst_id.clone(), updated_inst, |_, _| false, overwrite)
            .await
        {
            error!(
                "Failed to write config instance '{}' to cache: {}",
                cfg_inst_id, e
            );
            continue;
        }
    }

    Ok(())
}
