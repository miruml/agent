// standard library
use std::env;

// external
use serde_json::json;

pub const PATH_DELIMITER: &str = "__SEP__";
pub const GIT_RELEASE_TAG_KEY: Option<&str> = option_env!("MIRU_AGENT_GIT_RELEASE_TAG");
pub const GIT_COMMIT_HASH_KEY: Option<&str> = option_env!("MIRU_AGENT_GIT_COMMIT_HASH");

pub fn time_delta_to_positive_duration(time_delta: chrono::Duration) -> std::time::Duration {
    if time_delta.num_milliseconds() <= 0 {
        std::time::Duration::from_secs(0)
    } else {
        std::time::Duration::from_millis(time_delta.num_milliseconds() as u64)
    }
}

pub fn has_version_flag() -> bool {
    let args: Vec<String> = env::args().collect();
    args.iter().any(|arg| arg == "version" || arg == "--version" || arg == "-v")
}

pub fn version_info() -> serde_json::Value {
    json!({
        "version": GIT_RELEASE_TAG_KEY.unwrap_or("unknown"),
        "commit": GIT_COMMIT_HASH_KEY.unwrap_or("unknown"),
    })
}
