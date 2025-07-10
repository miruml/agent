use crate::http::config_instances::ConfigInstancesExt;
use crate::models::config_instance::{ActivityStatus, ErrorStatus};
use crate::storage::config_instances::ConfigInstanceCache;
use crate::sync::errors::{SyncCacheErr, SyncErr};
use crate::trace;
use openapi_client::models::UpdateConfigInstanceRequest;

// external crates
use tracing::{debug, error};

pub async fn push_config_instances<HTTPClientT: ConfigInstancesExt>(
    cfg_inst_cache: &ConfigInstanceCache,
    http_client: &HTTPClientT,
    token: &str,
) -> Result<(), SyncErr> {
    // get all unsynced instances
    let unsynced_entries = cfg_inst_cache.get_dirty_entries().await.map_err(|e| {
        SyncErr::CacheErr(Box::new(SyncCacheErr {
            source: e,
            trace: trace!(),
        }))
    })?;
    debug!(
        "Found {} unsynced config instances: {:?}",
        unsynced_entries.len(),
        unsynced_entries
    );

    // push each unsynced config instance to the server and update the cache
    for entry in unsynced_entries {
        let inst = entry.value;

        // define the updates
        let activity_status = ActivityStatus::to_backend(&inst.activity_status);
        let error_status = ErrorStatus::to_backend(&inst.error_status);
        let updates = UpdateConfigInstanceRequest {
            activity_status: Some(activity_status),
            error_status: Some(error_status),
        };

        // send to the server
        debug!(
            "Pushing config instance {} to the server with updates: {:?}",
            inst.id, updates
        );
        if let Err(e) = http_client
            .update_config_instance(&inst.id, &updates, token)
            .await
        {
            error!("Failed to push config instance {}: {}", inst.id, e);
        }

        // update the cache
        debug!("Updating cache for config instance {}", inst.id);
        let inst_id = inst.id.clone();
        if let Err(e) = cfg_inst_cache
            .write(inst.id.clone(), inst, |_, _| false, true)
            .await
        {
            error!(
                "Failed to update cache for config instance {} after pushing to the server: {}",
                inst_id, e
            );
        }
    }

    Ok(())
}
