// internal crates
use crate::filesys::{cached_file::ConcurrentCachedFile, file::File, path::PathExt};
use crate::models::{device, device::Device};
use crate::storage::errors::{DeviceNotActivatedErr, StorageErr, StorageFileSysErr};
use crate::trace;

pub type DeviceFile = ConcurrentCachedFile<Device, device::Updates>;

pub async fn assert_activated(device_file: &File) -> Result<(), StorageErr> {
    // check the agent file exists
    device_file.assert_exists().map_err(|e| {
        StorageErr::FileSysErr(Box::new(StorageFileSysErr {
            source: e,
            trace: trace!(),
        }))
    })?;

    // attempt to read it
    let device = device_file.read_json::<Device>().await.map_err(|e| {
        StorageErr::FileSysErr(Box::new(StorageFileSysErr {
            source: e,
            trace: trace!(),
        }))
    })?;

    // check the agent is activated
    if !device.activated {
        return Err(StorageErr::DeviceNotActivatedErr(Box::new(
            DeviceNotActivatedErr {
                msg: "device is not activated".to_string(),
                trace: trace!(),
            },
        )));
    }

    Ok(())
}
