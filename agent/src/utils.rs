pub const PATH_DELIMITER: &str = "__SEP__";

pub fn time_delta_to_positive_duration(time_delta: chrono::Duration) -> std::time::Duration {
    if time_delta.num_milliseconds() <= 0 {
        std::time::Duration::from_secs(0)
    } else {
        std::time::Duration::from_millis(time_delta.num_milliseconds() as u64)
    }
}
