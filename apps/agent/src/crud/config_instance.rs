// internal crates
use crate::fsm::config_instance as fsm;
use crate::models::config_instance::{
    ConfigInstance,
    ConfigInstanceActivityStatus,
};

pub fn matches_config_schema_and_activity_status(
    instance: &ConfigInstance,
    config_schema_id: &str,
    activity_status: ConfigInstanceActivityStatus,
) -> bool {
    instance.config_schema_id == config_schema_id && instance.activity_status == activity_status
}

pub fn matches_filepath_and_activity_status(
    instance: &ConfigInstance,
    filepath: &str,
    status: ConfigInstanceActivityStatus,
) -> bool {
    let instance_filepath = match &instance.filepath {
        Some(filepath) => filepath,
        None => return false,
    };
    filepath == instance_filepath && status == instance.activity_status
}

pub fn matches_config_schema_and_next_action(
    instance: &ConfigInstance,
    config_schema_id: &str,
    next_action: fsm::NextAction,
    use_cooldown: bool,
) -> bool {
    instance.config_schema_id == config_schema_id &&
    fsm::next_action(&instance, use_cooldown) == next_action
}