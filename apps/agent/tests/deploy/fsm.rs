use config_agent::deploy::fsm;
use config_agent::errors::MiruError;
use config_agent::models::config_instance::{
    ActivityStatus, ConfigInstance, ErrorStatus, TargetStatus,
};

use crate::mock::MockMiruError;

// external crates
use chrono::{TimeDelta, Utc};

// ================================= NEXT ACTION =================================== //
pub mod next_action {

    use super::*;

    fn validate_eq_wait_time(expected: TimeDelta, actual: TimeDelta, tol: TimeDelta) {
        assert!(
            expected - actual > -tol,
            "expected wait time {} is not equal to actual wait time {}",
            expected,
            actual
        );
        assert!(
            expected - actual < tol,
            "expected wait time {} is not equal to actual wait time {}",
            expected,
            actual
        );
    }

    fn validate_next_action(expected: fsm::NextAction, actual: fsm::NextAction) {
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
            expected_wait_time,
            actual_wait_time,
            TimeDelta::milliseconds(1),
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
        validate_next_action(target_created, fsm::next_action(&instance, use_cooldown));
        instance.target_status = TargetStatus::Deployed;
        validate_next_action(target_deployed, fsm::next_action(&instance, use_cooldown));
        instance.target_status = TargetStatus::Removed;
        validate_next_action(target_removed, fsm::next_action(&instance, use_cooldown));
    }

    #[test]
    fn created_activity_status() {
        let mut instance = ConfigInstance {
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

    #[test]
    fn queued_activity_status() {
        let mut instance = ConfigInstance {
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

    #[test]
    fn deployed_activity_status() {
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

    #[test]
    fn removed_activity_status() {
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

#[test]
fn is_action_required() {
    assert!(!fsm::is_action_required(fsm::NextAction::None));
    assert!(fsm::is_action_required(fsm::NextAction::Deploy));
    assert!(fsm::is_action_required(fsm::NextAction::Remove));
    assert!(!fsm::is_action_required(fsm::NextAction::Wait(
        TimeDelta::minutes(1)
    )));
}

// ================================= TRANSITIONS =================================== //
#[test]
fn calc_exp_backoff() {
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

pub mod transitions {
    use super::*;

    fn def_deps_w_all_status_combos() -> Vec<ConfigInstance> {
        let mut instances = Vec::new();
        for activity in ActivityStatus::variants() {
            for target in TargetStatus::variants() {
                for error in ErrorStatus::variants() {
                    instances.push(ConfigInstance {
                        activity_status: activity,
                        target_status: target,
                        error_status: error,
                        ..Default::default()
                    });
                }
            }
        }

        instances
    }

    fn def_deps_w_error_status(error_status: ErrorStatus) -> Vec<ConfigInstance> {
        let mut instances = Vec::new();
        for activity in ActivityStatus::variants() {
            for target in TargetStatus::variants() {
                instances.push(ConfigInstance {
                    activity_status: activity,
                    target_status: target,
                    error_status,
                    ..Default::default()
                });
            }
        }

        instances
    }

    fn validate_deploy_transition(instance: ConfigInstance, expected_error_status: ErrorStatus) {
        let actual = fsm::deploy(instance.clone());

        let expected = ConfigInstance {
            id: instance.id.clone(),
            target_status: instance.target_status,
            activity_status: ActivityStatus::Deployed,
            error_status: expected_error_status,
            relative_filepath: instance.relative_filepath.clone(),
            patch_id: instance.patch_id.clone(),
            created_by_id: instance.created_by_id.clone(),
            created_at: instance.created_at,
            updated_by_id: instance.updated_by_id.clone(),
            updated_at: instance.updated_at,
            device_id: instance.device_id.clone(),
            config_schema_id: instance.config_schema_id.clone(),
            attempts: 0,
            cooldown_ends_at: actual.cooldown_ends_at,
        };
        assert!(
            expected == actual,
            "expected:\n{:?}\n actual:\n{:?}\n",
            expected,
            actual
        );

        // check the cooldown
        assert!(instance.cooldown_ends_at < Utc::now());
    }

    #[test]
    fn deploy_error_status_none() {
        let instances = def_deps_w_error_status(ErrorStatus::None);
        for instance in instances {
            validate_deploy_transition(instance, ErrorStatus::None);
        }
    }

    #[test]
    fn deploy_error_status_retrying() {
        let instances = def_deps_w_error_status(ErrorStatus::Retrying);
        for instance in instances {
            match instance.target_status {
                TargetStatus::Deployed => validate_deploy_transition(instance, ErrorStatus::None),
                _ => validate_deploy_transition(instance, ErrorStatus::Retrying),
            }
        }
    }

    #[test]
    fn deploy_error_status_failed() {
        let instances = def_deps_w_error_status(ErrorStatus::Failed);
        for instance in instances {
            validate_deploy_transition(instance, ErrorStatus::Failed);
        }
    }

    fn validate_remove_transition(instance: ConfigInstance, expected_error_status: ErrorStatus) {
        let actual = fsm::remove(instance.clone());

        let expected = ConfigInstance {
            id: instance.id.clone(),
            target_status: instance.target_status,
            activity_status: ActivityStatus::Removed,
            error_status: expected_error_status,
            relative_filepath: instance.relative_filepath.clone(),
            patch_id: instance.patch_id.clone(),
            created_by_id: instance.created_by_id.clone(),
            created_at: instance.created_at,
            updated_by_id: instance.updated_by_id.clone(),
            updated_at: instance.updated_at,
            device_id: instance.device_id.clone(),
            config_schema_id: instance.config_schema_id.clone(),
            attempts: 0,
            cooldown_ends_at: actual.cooldown_ends_at,
        };
        assert!(
            expected == actual,
            "expected:\n{:?}\n actual:\n{:?}\n",
            expected,
            actual
        );

        // check the cooldown
        assert!(instance.cooldown_ends_at < Utc::now());
    }

    #[test]
    fn remove_error_status_none() {
        let instances = def_deps_w_error_status(ErrorStatus::None);
        for instance in instances {
            validate_remove_transition(instance, ErrorStatus::None);
        }
    }

    #[test]
    fn remove_error_status_retrying() {
        let instances = def_deps_w_error_status(ErrorStatus::Retrying);
        for instance in instances {
            match instance.target_status {
                TargetStatus::Removed | TargetStatus::Created => {
                    validate_remove_transition(instance, ErrorStatus::None)
                }
                _ => validate_remove_transition(instance, ErrorStatus::Retrying),
            }
        }
    }

    #[test]
    fn remove_error_status_failed() {
        let instances = def_deps_w_error_status(ErrorStatus::Failed);
        for instance in instances {
            validate_remove_transition(instance, ErrorStatus::Failed);
        }
    }

    fn validate_error_transition(
        instance: ConfigInstance,
        settings: &fsm::Settings,
        e: &impl MiruError,
        increment_attempts: bool,
    ) {
        let attempts = if increment_attempts && !e.is_network_connection_error() {
            instance.attempts + 1
        } else {
            instance.attempts
        };
        let expected_err_status =
            if attempts >= settings.max_attempts || instance.error_status == ErrorStatus::Failed {
                ErrorStatus::Failed
            } else {
                ErrorStatus::Retrying
            };
        let actual = fsm::error(instance.clone(), settings, e, increment_attempts);

        let expected = ConfigInstance {
            id: instance.id.clone(),
            target_status: instance.target_status,
            activity_status: instance.activity_status,
            error_status: expected_err_status,
            relative_filepath: instance.relative_filepath.clone(),
            patch_id: instance.patch_id.clone(),
            created_by_id: instance.created_by_id.clone(),
            created_at: instance.created_at,
            updated_by_id: instance.updated_by_id.clone(),
            updated_at: instance.updated_at,
            device_id: instance.device_id.clone(),
            config_schema_id: instance.config_schema_id.clone(),
            attempts,
            cooldown_ends_at: actual.cooldown_ends_at,
        };
        assert!(
            expected == actual,
            "expected:\n{:?}\n actual:\n{:?}\n",
            expected,
            actual
        );

        // check the cooldown
        let now = Utc::now();
        let cooldown = fsm::calc_exp_backoff(
            2,
            settings.exp_backoff_base_secs,
            attempts,
            settings.max_cooldown_secs,
        );
        let expected_cooldown_ends_at = now + TimeDelta::seconds(cooldown as i64);
        assert!(
            actual.cooldown_ends_at <= expected_cooldown_ends_at,
            "actual:\n{:?}\n expected:\n{:?}\n",
            actual.cooldown_ends_at,
            expected_cooldown_ends_at
        );
        assert!(
            actual.cooldown_ends_at >= expected_cooldown_ends_at - TimeDelta::seconds(1),
            "actual:\n{:?}\n expected:\n{:?}\n",
            actual.cooldown_ends_at,
            expected_cooldown_ends_at
        );
    }

    #[test]
    fn error_transition() {
        let settings = fsm::Settings {
            max_attempts: 5,
            exp_backoff_base_secs: 1,
            max_cooldown_secs: 60,
        };

        for i in 0..4 {
            let network_err = i % 2 == 0;
            let increment_attempts = i < 3;

            // no failed attempts
            let instances = def_deps_w_all_status_combos();
            for mut instance in instances {
                instance.attempts = 0;
                validate_error_transition(
                    instance.clone(),
                    &settings,
                    &MockMiruError::new(network_err),
                    increment_attempts,
                );
            }

            // failed attempts not reached max
            let instances = def_deps_w_all_status_combos();
            for mut instance in instances {
                instance.attempts = settings.max_attempts - 2;
                validate_error_transition(
                    instance.clone(),
                    &settings,
                    &MockMiruError::new(network_err),
                    increment_attempts,
                );
            }

            // failed attempts reached max
            let instances = def_deps_w_all_status_combos();
            for mut instance in instances {
                instance.attempts = settings.max_attempts - 1;
                validate_error_transition(
                    instance.clone(),
                    &settings,
                    &MockMiruError::new(network_err),
                    increment_attempts,
                );
            }

            // failed attempts exceeding max
            let instances = def_deps_w_all_status_combos();
            for mut instance in instances {
                instance.attempts = settings.max_attempts + 1;
                validate_error_transition(
                    instance.clone(),
                    &settings,
                    &MockMiruError::new(network_err),
                    increment_attempts,
                );
            }
        }
    }
}
