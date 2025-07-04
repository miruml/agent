// standard crates
use std::time::Duration;

// internal crates
use config_agent::utils::{as_duration, calc_exp_backoff};

// external crates
use chrono::TimeDelta;

#[test]
fn test_as_duration() {
    // positive time delta
    let time_delta = TimeDelta::seconds(10);
    let duration = as_duration(time_delta);
    assert_eq!(duration, Duration::from_secs(10));

    // negative time delta
    let time_delta = TimeDelta::seconds(-10);
    let duration = as_duration(time_delta);
    assert_eq!(duration, Duration::from_secs(0));
}

#[test]
fn test_calc_exp_backoff() {
    // base = 1
    assert_eq!(calc_exp_backoff(2, 1, 0, 10), 2);
    assert_eq!(calc_exp_backoff(4, 1, 1, 10), 4);
    assert_eq!(calc_exp_backoff(11, 1, 2, 10), 10);

    // base = 2
    assert_eq!(calc_exp_backoff(1, 2, 0, 10), 1);
    assert_eq!(calc_exp_backoff(1, 2, 1, 10), 2);
    assert_eq!(calc_exp_backoff(1, 2, 3, 10), 8);
    assert_eq!(calc_exp_backoff(1, 2, 4, 10), 10);

    // base = 4
    assert_eq!(calc_exp_backoff(3, 4, 0, 56), 3);
    assert_eq!(calc_exp_backoff(3, 4, 1, 56), 12);
    assert_eq!(calc_exp_backoff(3, 4, 2, 56), 48);
    assert_eq!(calc_exp_backoff(3, 4, 3, 56), 56);
}
