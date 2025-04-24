pub const PATH_DELIMITER: &str = "__SEP__";

pub fn time_delta_to_duration(time_delta: chrono::Duration) -> std::time::Duration {
    std::time::Duration::from_secs(time_delta.num_seconds().max(0) as u64)
        + std::time::Duration::from_nanos(time_delta.subsec_nanos() as u64)
}
