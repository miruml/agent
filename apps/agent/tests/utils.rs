// standard crates
use std::time::Duration;

// internal crates
use config_agent::utils::time_delta_to_duration;

// external crates
use chrono::TimeDelta;



#[test]
fn test_time_delta_to_duration() {

    // positive time delta
    let time_delta = TimeDelta::seconds(10);
    let duration = time_delta_to_duration(time_delta);
    assert_eq!(duration, Duration::from_secs(10));

    // negative time delta
    let time_delta = TimeDelta::seconds(-10);
    let duration = time_delta_to_duration(time_delta);
    assert_eq!(duration, Duration::from_secs(0));
}

