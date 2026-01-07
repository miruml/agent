use crate::http::devices::DevicesExt;
use crate::models::device;
use crate::storage::device::DeviceFile;
use crate::sync::errors::*;
use crate::trace;

use tracing::info;

pub async fn push<HTTPClientT: DevicesExt>(
    device_file: &DeviceFile,
    http_client: &HTTPClientT,
    token: &str,
    agent_version: String,
) -> Result<(), SyncErr> {
    let device = device_file.read().await.map_err(|e| {
        SyncErr::FileSysErr(Box::new(SyncFileSysErr {
            source: e,
            trace: trace!(),
        }))
    })?;
    if device.agent_version == agent_version {
        return Ok(());
    }

    info!(
        "Detected new agent version: {} -> {}",
        device.agent_version, agent_version
    );

    // update the device file
    let updates = device::Updates {
        agent_version: Some(agent_version.clone()),
        ..device::Updates::empty()
    };

    device_file.patch(updates).await.map_err(|e| {
        SyncErr::FileSysErr(Box::new(SyncFileSysErr {
            source: e,
            trace: trace!(),
        }))
    })?;

    // update the backend
    http_client
        .update_device(
            &device.id,
            &openapi_client::models::UpdateDeviceFromAgentRequest {
                agent_version: Some(agent_version),
            },
            token,
        )
        .await
        .map_err(|e| {
            SyncErr::HTTPClientErr(Box::new(SyncHTTPClientErr {
                source: e,
                trace: trace!(),
            }))
        })?;

    Ok(())
}
