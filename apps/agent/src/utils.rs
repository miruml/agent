// standard library
use std::cmp::min;
use std::env;

// external
use serde_json::json;

// globals
pub const PATH_DELIMITER: &str = "__SEP__";
pub const GIT_RELEASE_TAG_KEY: Option<&str> = option_env!("MIRU_AGENT_GIT_RELEASE_TAG");
pub const GIT_COMMIT_HASH_KEY: Option<&str> = option_env!("MIRU_AGENT_GIT_COMMIT_HASH");

pub fn as_duration(time_delta: chrono::TimeDelta) -> std::time::Duration {
    if time_delta.num_milliseconds() <= 0 {
        std::time::Duration::from_secs(0)
    } else {
        std::time::Duration::from_millis(time_delta.num_milliseconds() as u64)
    }
}

pub fn has_version_flag() -> bool {
    let args: Vec<String> = env::args().collect();
    args.iter()
        .any(|arg| arg == "version" || arg == "--version" || arg == "-v")
}

pub fn version_info() -> serde_json::Value {
    json!({
        "version": GIT_RELEASE_TAG_KEY.unwrap_or("unknown"),
        "commit": GIT_COMMIT_HASH_KEY.unwrap_or("unknown"),
    })
}

pub fn calc_exp_backoff(base: i64, growth_factor: i64, exp: u32, max: i64) -> i64 {
    min(base.saturating_mul(growth_factor.saturating_pow(exp)), max)
}

#[derive(Debug, Clone, Copy)]
pub struct CooldownOptions {
    pub base_secs: i64,
    pub growth_factor: i64,
    pub max_secs: i64,
}

impl Default for CooldownOptions {
    fn default() -> Self {
        Self {
            base_secs: 15,
            growth_factor: 2,
            max_secs: 12 * 60 * 60, // 12 hours
        }
    }
}
