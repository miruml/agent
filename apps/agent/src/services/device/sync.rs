// internal crates
use crate::sync::{errors::*, syncer::SyncerExt};
use crate::trace;
use crate::{errors::MiruError, services::errors::*};
use openapi_server::models::{SyncDeviceResponse, SyncDeviceResult};


pub async fn sync_device<SyncerT: SyncerExt>(
    syncer: &SyncerT,
) -> Result<SyncDeviceResponse, ServiceErr> {
    match syncer.sync().await {
        Ok(()) => {
            let sync_state = syncer.get_sync_state().await.map_err(|e| {
                ServiceErr::SyncErr(Box::new(ServiceSyncErr {
                    source: e,
                    trace: trace!(),
                }))
            })?;
            Ok(SyncDeviceResponse {
                code: SyncDeviceResult::SYNC_DEVICE_RESULT_SUCCESS,
                message: "successfully synced".to_string(),
                last_synced_at: sync_state.last_synced_at.to_rfc3339(),
                last_attempted_sync_at: sync_state.last_attempted_sync_at.to_rfc3339(),
                is_cooling_down: sync_state.is_in_cooldown(),
                cooldown_ends_at: sync_state.cooldown_ends_at.to_rfc3339(),
            })
        }
        Err(e) => {
            let mut code = SyncDeviceResult::SYNC_DEVICE_RESULT_INTERNAL_SERVER_ERROR;
            if matches!(e, SyncErr::InCooldownErr(_)) {
                code = SyncDeviceResult::SYNC_DEVICE_RESULT_DEVICE_IN_COOLDOWN;
            } else if e.is_network_connection_error() {
                code = SyncDeviceResult::SYNC_DEVICE_RESULT_NETWORK_CONNECTION_ERROR;
            }

            let sync_state = syncer.get_sync_state().await.map_err(|e| {
                ServiceErr::SyncErr(Box::new(ServiceSyncErr {
                    source: e,
                    trace: trace!(),
                }))
            })?;

            Ok(SyncDeviceResponse {
                code,
                message: e.to_string(),
                last_synced_at: sync_state.last_synced_at.to_rfc3339(),
                last_attempted_sync_at: sync_state.last_attempted_sync_at.to_rfc3339(),
                is_cooling_down: sync_state.is_in_cooldown(),
                cooldown_ends_at: sync_state.cooldown_ends_at.to_rfc3339(),
            })
        }
    }
}
