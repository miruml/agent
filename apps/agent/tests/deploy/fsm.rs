use config_agent::models::config_instance::{
    ConfigInstance,
    ActivityStatus,
    TargetStatus,
    ErrorStatus,
};
use config_agent::deploy::fsm;

// external crates
use chrono::TimeDelta;

// ================================= NEXT ACTION =================================== //
pub mod next_action {

    use super::*;

    fn validate_eq_wait_time(
        expected: TimeDelta,
        actual: TimeDelta,
        tol: TimeDelta,
    ) {
        assert!(
            expected - actual > -tol,
            "expected wait time {} is not equal to actual wait time {}",
            expected, actual
        );
        assert!(
            expected - actual < tol,
            "expected wait time {} is not equal to actual wait time {}",
            expected, actual
        );
    }

    fn validate_next_action(
        expected: fsm::NextAction,
        actual: fsm::NextAction,
    ) {
        // if the expected action is not a wait, then compare the actions
        let expected_wait_time = match expected {
            fsm::NextAction::Wait(expected_wait_time) => expected_wait_time,
            _ => {
                assert_eq!(expected, actual);
                return;
            }
        };

        // if the actual action is not a wait, then compare the actions
        let actual_wait_time = match actual {
            fsm::NextAction::Wait(actual_wait_time) => actual_wait_time,
            _ => {
                assert_eq!(expected, actual);
                return;
            }
        };

        // both actions are a wait, so compare the wait times
        validate_eq_wait_time(
            expected_wait_time, actual_wait_time, TimeDelta::milliseconds(1),
        );
    }

    fn validate_next_actions(
        mut instance: ConfigInstance,
        use_cooldown: bool,
        target_created: fsm::NextAction,
        target_deployed: fsm::NextAction,
        target_removed: fsm::NextAction,
    ) {
        instance.target_status = TargetStatus::Created;
        validate_next_action(
            target_created,
            fsm::next_action(&instance, use_cooldown),
        );
        instance.target_status = TargetStatus::Deployed;
        validate_next_action(
            target_deployed,
            fsm::next_action(&instance, use_cooldown),
        );
        instance.target_status = TargetStatus::Removed;
        validate_next_action(
            target_removed,
            fsm::next_action(&instance, use_cooldown),
        );
    }

    #[tokio::test]
    async fn created_activity_status() {
        let mut instance= ConfigInstance {
            activity_status: ActivityStatus::Created,
            error_status: ErrorStatus::None,
            ..Default::default()
        };

        // error status 'None' or 'Retrying' && not in cooldown
        for i in 0..2 {
            if i == 0 {
                instance.error_status = ErrorStatus::None;
            } else {
                instance.error_status = ErrorStatus::Retrying;
            }
            validate_next_actions(
                instance.clone(),
                true,
                fsm::NextAction::None,
                fsm::NextAction::Deploy,
                fsm::NextAction::Remove,
            );
        }

        // error status 'None' or 'Retrying' && in cooldown
        for i in 0..2 {
            if i == 0 {
                instance.error_status = ErrorStatus::None;
            } else {
                instance.error_status = ErrorStatus::Retrying;
            }
            let cooldown = TimeDelta::minutes(60);
            instance.set_cooldown(cooldown);

            // using cooldown
            validate_next_actions(
                instance.clone(),
                true,
                fsm::NextAction::Wait(cooldown),
                fsm::NextAction::Wait(cooldown),
                fsm::NextAction::Wait(cooldown),
            );

            // ignore cooldown
            validate_next_actions(
                instance.clone(),
                false,
                fsm::NextAction::None,
                fsm::NextAction::Deploy,
                fsm::NextAction::Remove,
            );
        }

        // error status 'Failed'
        instance.error_status = ErrorStatus::Failed;
        validate_next_actions(
            instance.clone(),
            true,
            fsm::NextAction::None,
            fsm::NextAction::None,
            fsm::NextAction::None,
        );
    }

    #[tokio::test]
    async fn queued_activity_status() {
        let mut instance= ConfigInstance {
            activity_status: ActivityStatus::Queued,
            error_status: ErrorStatus::None,
            ..Default::default()
        };

        // error status 'None' or 'Retrying' && not in cooldown
        for i in 0..2 {
            if i == 0 {
                instance.error_status = ErrorStatus::None;
            } else {
                instance.error_status = ErrorStatus::Retrying;
            }
            validate_next_actions(
                instance.clone(),
                true,
                fsm::NextAction::None,
                fsm::NextAction::Deploy,
                fsm::NextAction::Remove,
            );
        }

        // error status 'None' or 'Retrying' && in cooldown
        for i in 0..2 {
            if i == 0 {
                instance.error_status = ErrorStatus::None;
            } else {
                instance.error_status = ErrorStatus::Retrying;
            }
            let cooldown = TimeDelta::minutes(60);
            instance.set_cooldown(cooldown);

            // using cooldown
            validate_next_actions(
                instance.clone(),
                true,
                fsm::NextAction::Wait(cooldown),
                fsm::NextAction::Wait(cooldown),
                fsm::NextAction::Wait(cooldown),
            );

            // ignore cooldown
            validate_next_actions(
                instance.clone(),
                false,
                fsm::NextAction::None,
                fsm::NextAction::Deploy,
                fsm::NextAction::Remove,
            );
        }

        // error status 'Failed'
        instance.error_status = ErrorStatus::Failed;
        validate_next_actions(
            instance.clone(),
            true,
            fsm::NextAction::None,
            fsm::NextAction::None,
            fsm::NextAction::None,
        );
    }

    #[tokio::test]
    async fn deployed_activity_status() {
        let mut instance = ConfigInstance {
            activity_status: ActivityStatus::Deployed,
            error_status: ErrorStatus::None,
            ..Default::default()
        };

        // error status 'None' or 'Retrying' && not in cooldown
        for i in 0..2 {
            if i == 0 {
                instance.error_status = ErrorStatus::None;
            } else {
                instance.error_status = ErrorStatus::Retrying;
            }
            validate_next_actions(
                instance.clone(),
                true,
                fsm::NextAction::Remove,
                fsm::NextAction::None,
                fsm::NextAction::Remove,
            );
        }

        // error status 'None' or 'Retrying' && in cooldown
        for i in 0..2 {
            if i == 0 {
                instance.error_status = ErrorStatus::None;
            } else {
                instance.error_status = ErrorStatus::Retrying;
            }
            let cooldown = TimeDelta::minutes(60);
            instance.set_cooldown(cooldown);

            // using cooldown
            validate_next_actions(
                instance.clone(),
                true,
                fsm::NextAction::Wait(cooldown),
                fsm::NextAction::Wait(cooldown),
                fsm::NextAction::Wait(cooldown),
            );

            // ignore cooldown
            validate_next_actions(
                instance.clone(),
                false,
                fsm::NextAction::Remove,
                fsm::NextAction::None,
                fsm::NextAction::Remove,
            );
        }

        // error status 'Failed'
        instance.error_status = ErrorStatus::Failed;
        validate_next_actions(
            instance.clone(),
            true,
            fsm::NextAction::None,
            fsm::NextAction::None,
            fsm::NextAction::None,
        );
    }

    #[tokio::test]
    async fn removed_activity_status() {
        let mut instance = ConfigInstance {
            activity_status: ActivityStatus::Removed,
            error_status: ErrorStatus::None,
            ..Default::default()
        };

        // error status 'None' or 'Retrying' && not in cooldown
        for i in 0..2 {
            if i == 0 {
                instance.error_status = ErrorStatus::None;
            } else {
                instance.error_status = ErrorStatus::Retrying;
            }
            validate_next_actions(
                instance.clone(),
                true,
                fsm::NextAction::None,
                fsm::NextAction::Deploy,
                fsm::NextAction::None,
            );
        }

        // error status 'None' or 'Retrying' && in cooldown
        for i in 0..2 {
            if i == 0 {
                instance.error_status = ErrorStatus::None;
            } else {
                instance.error_status = ErrorStatus::Retrying;
            }
            let cooldown = TimeDelta::minutes(60);
            instance.set_cooldown(cooldown);

            // using cooldown
            validate_next_actions(
                instance.clone(),
                true,
                fsm::NextAction::Wait(cooldown),
                fsm::NextAction::Wait(cooldown),
                fsm::NextAction::Wait(cooldown),
            );

            // ignore cooldown
            validate_next_actions(
                instance.clone(),
                false,
                fsm::NextAction::None,
                fsm::NextAction::Deploy,
                fsm::NextAction::None,
            );
        }

        // error status 'Failed'
        instance.error_status = ErrorStatus::Failed;
        validate_next_actions(
            instance.clone(),
            true,
            fsm::NextAction::None,
            fsm::NextAction::None,
            fsm::NextAction::None,
        );
    }
}

#[tokio::test]
async fn is_action_required() {
    assert!(!fsm::is_action_required(fsm::NextAction::None));
    assert!(fsm::is_action_required(fsm::NextAction::Deploy));
    assert!(fsm::is_action_required(fsm::NextAction::Remove));
    assert!(!fsm::is_action_required(fsm::NextAction::Wait(TimeDelta::minutes(1))));
}

// ================================= TRANSITIONS =================================== //
#[tokio::test]
async fn calc_exp_backoff() {
    // base = 1
    assert_eq!(fsm::calc_exp_backoff(2, 1, 0, 10), 2);
    assert_eq!(fsm::calc_exp_backoff(4, 1, 1, 10), 4);
    assert_eq!(fsm::calc_exp_backoff(11, 1, 2, 10), 10);

    // base = 2
    assert_eq!(fsm::calc_exp_backoff(1, 2, 0, 10), 1);
    assert_eq!(fsm::calc_exp_backoff(1, 2, 1, 10), 2);
    assert_eq!(fsm::calc_exp_backoff(1, 2, 3, 10), 8);
    assert_eq!(fsm::calc_exp_backoff(1, 2, 4, 10), 10);

    // base = 4
    assert_eq!(fsm::calc_exp_backoff(3, 4, 0, 56), 3);
    assert_eq!(fsm::calc_exp_backoff(3, 4, 1, 56), 12);
    assert_eq!(fsm::calc_exp_backoff(3, 4, 2, 56), 48);
    assert_eq!(fsm::calc_exp_backoff(3, 4, 3, 56), 56);
}