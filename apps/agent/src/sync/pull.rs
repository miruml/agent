use crate::cache::entry::is_dirty_false;
use crate::crud::prelude::*;
use crate::http::{
    config_instances::{
        ActivityStatusFilter,
        ConfigInstanceFiltersBuilder,
        ConfigInstancesExt,
        IDFilter,
    },
    search::SearchOperator,
};
use crate::models::config_instance::{
    ConfigInstance,
    TargetStatus,
};
use crate::storage::config_instances::{
    ConfigInstanceCache,
    ConfigInstanceDataCache,
};
use crate::sync::errors::{
    SyncCrudErr,
    SyncErr,
    SyncHTTPClientErr,
    ConfigInstanceDataNotFoundErr,
};
use crate::trace;
use openapi_client::models::{
    BackendConfigInstance,
    ConfigInstanceActivityStatus,
    ConfigInstanceExpand,
};

// external crates
use tracing::error;


pub async fn pull_config_instances<HTTPClientT: ConfigInstancesExt>(
    cfg_inst_cache: &ConfigInstanceCache,
    cfg_inst_data_cache: &ConfigInstanceDataCache,
    http_client: &HTTPClientT,
    device_id: &str,
    token: &str,
) -> Result<(), SyncErr> {

    let unremoved_insts = fetch_unremoved_instances(
        http_client, device_id, token,
    ).await?;

    let categorized_insts = categorize_instances(
        cfg_inst_cache,
        unremoved_insts,
    ).await?;

    let unknown_insts = fetch_instances_with_expanded_instance_data(
        http_client,
        device_id,
        categorized_insts.unknown.iter().map(|inst| inst.id.clone()).collect(),
        token,
    ).await?;

    add_unknown_instances_to_storage(
        cfg_inst_cache,
        cfg_inst_data_cache,
        unknown_insts,
    ).await?;

    update_target_status_instances(
        cfg_inst_cache,
        categorized_insts.update_target_status,
    ).await?;

    Ok(())
}

pub async fn fetch_unremoved_instances<HTTPClientT: ConfigInstancesExt>(
    http_client: &HTTPClientT,
    device_id: &str,
    token: &str,
) -> Result<Vec<BackendConfigInstance>, SyncErr> {

    let filters = ConfigInstanceFiltersBuilder::new(
        device_id.to_string(),
    ).with_activity_status_filters(ActivityStatusFilter {
        not: true,
        op: SearchOperator::Equals,
        val: vec![ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_REMOVED],
    }).build();
    http_client.list_all_config_instances(
        filters, &[] as &[ConfigInstanceExpand], token,
    ).await.map_err(|e| {
        SyncErr::HTTPClientErr(SyncHTTPClientErr { source: e, trace: trace!() })
    })
}

pub struct CategorizedConfigInstances {
    pub unknown: Vec<BackendConfigInstance>,
    pub update_target_status: Vec<ConfigInstance>,
    pub other: Vec<BackendConfigInstance>,
}

pub async fn categorize_instances(
    cfg_inst_cache: &ConfigInstanceCache,
    unremoved_insts: Vec<BackendConfigInstance>,
) -> Result<CategorizedConfigInstances, SyncErr> {

    let mut categorized = CategorizedConfigInstances {
        unknown: Vec::new(),
        update_target_status: Vec::new(),
        other: Vec::new(),
    };

    // unknown config instances
    for server_inst in unremoved_insts {

        // check if the config instance is known
        let mut storage_inst = match cfg_inst_cache.read_optional(server_inst.id.clone()).await.map_err(|e| {
            SyncErr::CrudErr(SyncCrudErr {
                source: e,
                trace: trace!(),
            })
        })? {
            Some(storage_inst) => storage_inst,
            None => {
                categorized.unknown.push(server_inst);
                continue;
            }
        };

        // check if the target status matches
        if storage_inst.target_status != TargetStatus::from_backend(&server_inst.target_status) {
            storage_inst.target_status = TargetStatus::from_backend(&server_inst.target_status);
            categorized.update_target_status.push(storage_inst);
        } else {
            categorized.other.push(server_inst);
        }
    }

    Ok(categorized)
}

pub async fn fetch_instances_with_expanded_instance_data<HTTPClientT: ConfigInstancesExt>(
    http_client: &HTTPClientT,
    device_id: &str,
    ids: Vec<String>,
    token: &str,
) -> Result<Vec<BackendConfigInstance>, SyncErr> {

    // read the unknown config instances from the server with instance data expanded
    let filters = ConfigInstanceFiltersBuilder::new(
        device_id.to_string(),
    ).with_id_filters(IDFilter {
        not: false,
        op: SearchOperator::Equals,
        val: ids,
    }).build();
    http_client.list_all_config_instances(
        filters, [ConfigInstanceExpand::CONFIG_INSTANCE_EXPAND_INSTANCE], token,
    ).await.map_err(|e| {
        SyncErr::HTTPClientErr(SyncHTTPClientErr { source: e, trace: trace!() })
    })
}

pub async fn add_unknown_instances_to_storage(
    cfg_inst_cache: &ConfigInstanceCache,
    cfg_inst_data_cache: &ConfigInstanceDataCache,
    unknown_insts: Vec<BackendConfigInstance>,
) -> Result<(), SyncErr> {

    // add the unknown config instances to the cache
    for mut unknown_inst in unknown_insts {
        
        // throw an error since if the instance data isn't expanded for this one it
        // won't be expanded for any others and none of the instances will therefore
        // be added to the cache
        let instance_data = match unknown_inst.instance {
            Some(instance_data) => instance_data,
            None => {
                return Err(SyncErr::ConfigInstanceDataNotFound(ConfigInstanceDataNotFoundErr {
                    instance_id: unknown_inst.id.clone(),
                    trace: trace!(),
                }));
            }
        };
        unknown_inst.instance = None;

        let overwrite = true;
        if let Err(e) = cfg_inst_data_cache.write(
            unknown_inst.id.clone(), instance_data, is_dirty_false, overwrite,
        ).await {
            error!("Failed to write instance data to cache for instance '{}': {}", unknown_inst.id, e);
            continue;
        }

        let unknown_inst_id = unknown_inst.id.clone();
        let storage_inst = ConfigInstance::from_backend(unknown_inst);
        let overwrite = true;
        if let Err(e) = cfg_inst_cache.write(
            unknown_inst_id.clone(), storage_inst, is_dirty_false, overwrite,
        ).await {
            error!("Failed to write instance to cache for instance '{}': {}", unknown_inst_id, e);
            continue;
        }
    }
    Ok(())
}

pub async fn update_target_status_instances(
    cfg_inst_cache: &ConfigInstanceCache,
    update_target_status: Vec<ConfigInstance>,
) -> Result<(), SyncErr> {

    for instance in update_target_status {
        let instance_id = instance.id.clone();
        let overwrite = true;
        if let Err(e) = cfg_inst_cache.write(
            instance_id.clone(), instance, is_dirty_false, overwrite,
        ).await {
            error!("Failed to write instance to cache for instance '{}': {}", instance_id, e);
            continue;
        }
    }

    Ok(())
}