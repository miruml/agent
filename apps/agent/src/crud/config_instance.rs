// internal crates
use crate::models::config_instance::{ActivityStatus, ConfigInstance};

pub fn matches_config_schema_and_activity_status(
    instance: &ConfigInstance,
    config_schema_id: &str,
    activity_status: ActivityStatus,
) -> bool {
    instance.config_schema_id == config_schema_id && instance.activity_status == activity_status
}

pub fn matches_filepath_and_activity_status(
    instance: &ConfigInstance,
    rel_filepath: &str,
    status: ActivityStatus,
) -> bool {
    let instance_rel_filepath = match &instance.relative_filepath {
        Some(filepath) => filepath,
        None => return false,
    };
    rel_filepath == instance_rel_filepath && status == instance.activity_status
}
