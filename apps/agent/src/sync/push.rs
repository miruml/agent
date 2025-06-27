use crate::http::config_instances::ConfigInstancesExt;
use crate::models::config_instance::{
    convert_activity_status_storage_to_backend,
    convert_error_status_storage_to_backend,
};
use crate::storage::config_instances::ConfigInstanceCache;
use crate::storage::cache::is_dirty_false;
use crate::sync::errors::{SyncStorageErr, SyncErr};
use crate::trace;
use openapi_client::models::UpdateConfigInstanceRequest;

// external crates
use tracing::error;

pub async fn push_config_instances<HTTPClientT: ConfigInstancesExt>(
    cfg_inst_cache: &ConfigInstanceCache,
    http_client: &HTTPClientT,
    token: &str,
) -> Result<(), SyncErr> {

    // get all unsynced instances
    let unsynced_entries = cfg_inst_cache.find_all_entries(
        |entry| { entry.is_dirty }
    ).await.map_err(|e| SyncErr::StorageErr(SyncStorageErr {
        source: e,
        trace: trace!(),
    }))?;

    // push each unsynced instance to the server and update the cache
    for entry in unsynced_entries {

        let inst = entry.value;

        // define the updates
        let activity_status = convert_activity_status_storage_to_backend(&inst.activity_status);
        let error_status = convert_error_status_storage_to_backend(&inst.error_status);
        let updates = UpdateConfigInstanceRequest {
            activity_status: Some(activity_status),
            error_status: Some(error_status),
        };

        // send to the server
        if let Err(e) = http_client.update_config_instance(
            &inst.id, &updates, token,
        ).await {
            error!("Failed to push config instance {}: {}", inst.id, e);
        }

        // update the cache
        let inst_id = inst.id.clone();
        if let Err(e) = cfg_inst_cache.write(
            inst.id.clone(),
            inst,
            is_dirty_false,
            true,
        ).await.map_err(|e| SyncErr::StorageErr(SyncStorageErr {
            source: e,
            trace: trace!(),
        })) {
            error!("Failed to update cache for config instance {} after pushing to the server: {}", inst_id, e);
        }
    }

    Ok(())
}
