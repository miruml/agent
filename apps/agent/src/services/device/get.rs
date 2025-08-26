// internal crates
use crate::models::device::Device;
use crate::services::errors::*;
use crate::storage::device::DeviceFile;
use crate::trace;


pub async fn get_device(device_file: &DeviceFile) -> Result<Device, ServiceErr> {
    let device = device_file.read().await.map_err(|e| {
        ServiceErr::FileSysErr(Box::new(ServiceFileSysErr {
            source: e,
            trace: trace!(),
        }))
    })?;
    Ok((*device).clone())
}
