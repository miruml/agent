// internal crates
use config_agent::activity::ActivityTracker;

// external crates
use chrono::Utc;

#[test]
fn activity_tracker() {
    let before_init = Utc::now().timestamp() as u64;
    let activity_tracker = ActivityTracker::new();
    let after_init = Utc::now().timestamp() as u64;
    assert!(activity_tracker.last_touched() >= before_init);
    assert!(activity_tracker.last_touched() <= after_init);

    let before_touch = Utc::now().timestamp() as u64;
    activity_tracker.touch();
    let after_touch = Utc::now().timestamp() as u64;
    assert!(activity_tracker.last_touched() >= before_touch);
    assert!(activity_tracker.last_touched() <= after_touch);
}
