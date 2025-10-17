// internal crates
use crate::models::config_instance::{ActivityStatus, ConfigInstance};

pub fn matches_config_schema_and_activity_status(
    cfg_inst: &ConfigInstance,
    config_schema_id: &str,
    activity_status: ActivityStatus,
) -> bool {
    cfg_inst.config_schema_id == config_schema_id && cfg_inst.activity_status == activity_status
}

pub fn matches_filepath_and_activity_status(
    cfg_inst: &ConfigInstance,
    rel_filepath: &str,
    status: ActivityStatus,
) -> bool {
    rel_filepath == cfg_inst.relative_filepath && status == cfg_inst.activity_status
}
