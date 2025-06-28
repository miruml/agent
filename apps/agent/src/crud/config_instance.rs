// internal crates
use crate::models::config_instance::{
    ConfigInstance,
    ActivityStatus,
};

pub fn matches_config_schema_and_activity_status(
    instance: &ConfigInstance,
    config_schema_id: &str,
    activity_status: ActivityStatus,
) -> bool {
    instance.config_schema_id == config_schema_id && instance.activity_status == activity_status
}

pub fn matches_filepath_and_activity_status(
    instance: &ConfigInstance,
    filepath: &str,
    status: ActivityStatus,
) -> bool {
    let instance_filepath = match &instance.filepath {
        Some(filepath) => filepath,
        None => return false,
    };
    filepath == instance_filepath && status == instance.activity_status
}

