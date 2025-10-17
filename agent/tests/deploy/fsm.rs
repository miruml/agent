use miru_agent::deploy::fsm;
use miru_agent::errors::MiruError;
use miru_agent::models::config_instance::{
    ActivityStatus, ConfigInstance, ErrorStatus, TargetStatus,
};
use miru_agent::utils::calc_exp_backoff;

use crate::mock::MockMiruError;

// external crates
use chrono::{TimeDelta, Utc};

// ================================= NEXT ACTION =================================== //
pub mod next_action {

    use super::*;

    fn validate_eq_wait_time(expected: TimeDelta, actual: TimeDelta, tol: TimeDelta) {
        assert!(
            expected - actual > -tol,
            "expected wait time {expected} is not equal to actual wait time {actual}",
        );
        assert!(
            expected - actual < tol,
            "expected wait time {expected} is not equal to actual wait time {actual}",
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
        mut cfg_inst: ConfigInstance,
        use_cooldown: bool,
        target_created: fsm::NextAction,
        target_validated: fsm::NextAction,
        target_deployed: fsm::NextAction,
        target_removed: fsm::NextAction,
    ) {
        cfg_inst.target_status = TargetStatus::Created;
        validate_next_action(target_created, fsm::next_action(&cfg_inst, use_cooldown));
        cfg_inst.target_status = TargetStatus::Validated;
        validate_next_action(target_validated, fsm::next_action(&cfg_inst, use_cooldown));
        cfg_inst.target_status = TargetStatus::Deployed;
        validate_next_action(target_deployed, fsm::next_action(&cfg_inst, use_cooldown));
        cfg_inst.target_status = TargetStatus::Removed;
        validate_next_action(target_removed, fsm::next_action(&cfg_inst, use_cooldown));
    }

    #[test]
    fn created_activity_status() {
        let mut cfg_inst = ConfigInstance {
            activity_status: ActivityStatus::Created,
            error_status: ErrorStatus::None,
            ..Default::default()
        };

        // error status 'None' or 'Retrying' && not in cooldown
        for i in 0..2 {
            if i == 0 {
                cfg_inst.error_status = ErrorStatus::None;
            } else {
                cfg_inst.error_status = ErrorStatus::Retrying;
            }
            validate_next_actions(
                cfg_inst.clone(),
                true,
                fsm::NextAction::None,
                fsm::NextAction::None,
                fsm::NextAction::None,
                fsm::NextAction::Archive,
            );
        }

        // error status 'None' or 'Retrying' && in cooldown
        for i in 0..2 {
            if i == 0 {
                cfg_inst.error_status = ErrorStatus::None;
            } else {
                cfg_inst.error_status = ErrorStatus::Retrying;
            }
            let cooldown = TimeDelta::minutes(60);
            cfg_inst.set_cooldown(cooldown);

            // using cooldown
            validate_next_actions(
                cfg_inst.clone(),
                true,
                fsm::NextAction::Wait(cooldown),
                fsm::NextAction::Wait(cooldown),
                fsm::NextAction::Wait(cooldown),
                fsm::NextAction::Wait(cooldown),
            );

            // ignore cooldown
            validate_next_actions(
                cfg_inst.clone(),
                false,
                fsm::NextAction::None,
                fsm::NextAction::None,
                fsm::NextAction::None,
                fsm::NextAction::Archive,
            );
        }

        // error status 'Failed'
        cfg_inst.error_status = ErrorStatus::Failed;
        validate_next_actions(
            cfg_inst.clone(),
            true,
            fsm::NextAction::None,
            fsm::NextAction::None,
            fsm::NextAction::None,
            fsm::NextAction::None,
        );
    }

    #[test]
    fn validating_activity_status() {
        let mut cfg_inst = ConfigInstance {
            activity_status: ActivityStatus::Validating,
            error_status: ErrorStatus::None,
            ..Default::default()
        };

        // error status 'None' or 'Retrying' && not in cooldown
        for i in 0..2 {
            if i == 0 {
                cfg_inst.error_status = ErrorStatus::None;
            } else {
                cfg_inst.error_status = ErrorStatus::Retrying;
            }
            validate_next_actions(
                cfg_inst.clone(),
                true,
                fsm::NextAction::None,
                fsm::NextAction::None,
                fsm::NextAction::None,
                fsm::NextAction::Archive,
            );
        }

        // error status 'None' or 'Retrying' && in cooldown
        for i in 0..2 {
            if i == 0 {
                cfg_inst.error_status = ErrorStatus::None;
            } else {
                cfg_inst.error_status = ErrorStatus::Retrying;
            }
            let cooldown = TimeDelta::minutes(60);
            cfg_inst.set_cooldown(cooldown);

            // using cooldown
            validate_next_actions(
                cfg_inst.clone(),
                true,
                fsm::NextAction::Wait(cooldown),
                fsm::NextAction::Wait(cooldown),
                fsm::NextAction::Wait(cooldown),
                fsm::NextAction::Wait(cooldown),
            );

            // ignore cooldown
            validate_next_actions(
                cfg_inst.clone(),
                false,
                fsm::NextAction::None,
                fsm::NextAction::None,
                fsm::NextAction::None,
                fsm::NextAction::Archive,
            );
        }

        // error status 'Failed'
        cfg_inst.error_status = ErrorStatus::Failed;
        validate_next_actions(
            cfg_inst.clone(),
            true,
            fsm::NextAction::None,
            fsm::NextAction::None,
            fsm::NextAction::None,
            fsm::NextAction::None,
        );
    }

    #[test]
    fn validated_activity_status() {
        let mut cfg_inst = ConfigInstance {
            activity_status: ActivityStatus::Validated,
            error_status: ErrorStatus::None,
            ..Default::default()
        };

        // error status 'None' or 'Retrying' && not in cooldown
        for i in 0..2 {
            if i == 0 {
                cfg_inst.error_status = ErrorStatus::None;
            } else {
                cfg_inst.error_status = ErrorStatus::Retrying;
            }
            validate_next_actions(
                cfg_inst.clone(),
                true,
                fsm::NextAction::None,
                fsm::NextAction::None,
                fsm::NextAction::None,
                fsm::NextAction::Archive,
            );
        }

        // error status 'None' or 'Retrying' && in cooldown
        for i in 0..2 {
            if i == 0 {
                cfg_inst.error_status = ErrorStatus::None;
            } else {
                cfg_inst.error_status = ErrorStatus::Retrying;
            }
            let cooldown = TimeDelta::minutes(60);
            cfg_inst.set_cooldown(cooldown);

            // using cooldown
            validate_next_actions(
                cfg_inst.clone(),
                true,
                fsm::NextAction::Wait(cooldown),
                fsm::NextAction::Wait(cooldown),
                fsm::NextAction::Wait(cooldown),
                fsm::NextAction::Wait(cooldown),
            );

            // ignore cooldown
            validate_next_actions(
                cfg_inst.clone(),
                false,
                fsm::NextAction::None,
                fsm::NextAction::None,
                fsm::NextAction::None,
                fsm::NextAction::Archive,
            );
        }

        // error status 'Failed'
        cfg_inst.error_status = ErrorStatus::Failed;
        validate_next_actions(
            cfg_inst.clone(),
            true,
            fsm::NextAction::None,
            fsm::NextAction::None,
            fsm::NextAction::None,
            fsm::NextAction::None,
        );
    }

    #[test]
    fn queued_activity_status() {
        let mut cfg_inst = ConfigInstance {
            activity_status: ActivityStatus::Queued,
            error_status: ErrorStatus::None,
            ..Default::default()
        };

        // error status 'None' or 'Retrying' && not in cooldown
        for i in 0..2 {
            if i == 0 {
                cfg_inst.error_status = ErrorStatus::None;
            } else {
                cfg_inst.error_status = ErrorStatus::Retrying;
            }
            validate_next_actions(
                cfg_inst.clone(),
                true,
                fsm::NextAction::Archive,
                fsm::NextAction::Archive,
                fsm::NextAction::Deploy,
                fsm::NextAction::Archive,
            );
        }

        // error status 'None' or 'Retrying' && in cooldown
        for i in 0..2 {
            if i == 0 {
                cfg_inst.error_status = ErrorStatus::None;
            } else {
                cfg_inst.error_status = ErrorStatus::Retrying;
            }
            let cooldown = TimeDelta::minutes(60);
            cfg_inst.set_cooldown(cooldown);

            // using cooldown
            validate_next_actions(
                cfg_inst.clone(),
                true,
                fsm::NextAction::Wait(cooldown),
                fsm::NextAction::Wait(cooldown),
                fsm::NextAction::Wait(cooldown),
                fsm::NextAction::Wait(cooldown),
            );

            // ignore cooldown
            validate_next_actions(
                cfg_inst.clone(),
                false,
                fsm::NextAction::Archive,
                fsm::NextAction::Archive,
                fsm::NextAction::Deploy,
                fsm::NextAction::Archive,
            );
        }

        // error status 'Failed'
        cfg_inst.error_status = ErrorStatus::Failed;
        validate_next_actions(
            cfg_inst.clone(),
            true,
            fsm::NextAction::None,
            fsm::NextAction::None,
            fsm::NextAction::None,
            fsm::NextAction::None,
        );
    }

    #[test]
    fn deployed_activity_status() {
        let mut cfg_inst = ConfigInstance {
            activity_status: ActivityStatus::Deployed,
            error_status: ErrorStatus::None,
            ..Default::default()
        };

        // error status 'None' or 'Retrying' && not in cooldown
        for i in 0..2 {
            if i == 0 {
                cfg_inst.error_status = ErrorStatus::None;
            } else {
                cfg_inst.error_status = ErrorStatus::Retrying;
            }
            validate_next_actions(
                cfg_inst.clone(),
                true,
                fsm::NextAction::Remove,
                fsm::NextAction::Remove,
                fsm::NextAction::None,
                fsm::NextAction::Remove,
            );
        }

        // error status 'None' or 'Retrying' && in cooldown
        for i in 0..2 {
            if i == 0 {
                cfg_inst.error_status = ErrorStatus::None;
            } else {
                cfg_inst.error_status = ErrorStatus::Retrying;
            }
            let cooldown = TimeDelta::minutes(60);
            cfg_inst.set_cooldown(cooldown);

            // using cooldown
            validate_next_actions(
                cfg_inst.clone(),
                true,
                fsm::NextAction::Wait(cooldown),
                fsm::NextAction::Wait(cooldown),
                fsm::NextAction::Wait(cooldown),
                fsm::NextAction::Wait(cooldown),
            );

            // ignore cooldown
            validate_next_actions(
                cfg_inst.clone(),
                false,
                fsm::NextAction::Remove,
                fsm::NextAction::Remove,
                fsm::NextAction::None,
                fsm::NextAction::Remove,
            );
        }

        // error status 'Failed'
        cfg_inst.error_status = ErrorStatus::Failed;
        validate_next_actions(
            cfg_inst.clone(),
            true,
            fsm::NextAction::None,
            fsm::NextAction::None,
            fsm::NextAction::None,
            fsm::NextAction::None,
        );
    }

    #[test]
    fn removed_activity_status() {
        let mut cfg_inst = ConfigInstance {
            activity_status: ActivityStatus::Removed,
            error_status: ErrorStatus::None,
            ..Default::default()
        };

        // error status 'None' or 'Retrying' && not in cooldown
        for i in 0..2 {
            if i == 0 {
                cfg_inst.error_status = ErrorStatus::None;
            } else {
                cfg_inst.error_status = ErrorStatus::Retrying;
            }
            validate_next_actions(
                cfg_inst.clone(),
                true,
                fsm::NextAction::None,
                fsm::NextAction::None,
                fsm::NextAction::Deploy,
                fsm::NextAction::None,
            );
        }

        // error status 'None' or 'Retrying' && in cooldown
        for i in 0..2 {
            if i == 0 {
                cfg_inst.error_status = ErrorStatus::None;
            } else {
                cfg_inst.error_status = ErrorStatus::Retrying;
            }
            let cooldown = TimeDelta::minutes(60);
            cfg_inst.set_cooldown(cooldown);

            // using cooldown
            validate_next_actions(
                cfg_inst.clone(),
                true,
                fsm::NextAction::Wait(cooldown),
                fsm::NextAction::Wait(cooldown),
                fsm::NextAction::Wait(cooldown),
                fsm::NextAction::Wait(cooldown),
            );

            // ignore cooldown
            validate_next_actions(
                cfg_inst.clone(),
                false,
                fsm::NextAction::None,
                fsm::NextAction::None,
                fsm::NextAction::Deploy,
                fsm::NextAction::None,
            );
        }

        // error status 'Failed'
        cfg_inst.error_status = ErrorStatus::Failed;
        validate_next_actions(
            cfg_inst.clone(),
            true,
            fsm::NextAction::None,
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

    fn validate_deploy_transition(cfg_inst: ConfigInstance, expected_error_status: ErrorStatus) {
        let actual = fsm::deploy(cfg_inst.clone());

        let expected = ConfigInstance {
            id: cfg_inst.id.clone(),
            target_status: cfg_inst.target_status,
            activity_status: ActivityStatus::Deployed,
            error_status: expected_error_status,
            relative_filepath: cfg_inst.relative_filepath.clone(),
            patch_id: cfg_inst.patch_id.clone(),
            created_at: cfg_inst.created_at,
            updated_at: cfg_inst.updated_at,
            device_id: cfg_inst.device_id.clone(),
            config_schema_id: cfg_inst.config_schema_id.clone(),
            config_type_id: cfg_inst.config_type_id.clone(),
            attempts: 0,
            cooldown_ends_at: actual.cooldown_ends_at,
        };
        assert!(
            expected == actual,
            "expected:\n{expected:?}\n actual:\n{actual:?}\n",
        );

        // check the cooldown
        assert!(cfg_inst.cooldown_ends_at < Utc::now());
    }

    #[test]
    fn deploy_error_status_none() {
        let instances = def_deps_w_error_status(ErrorStatus::None);
        for cfg_inst in instances {
            validate_deploy_transition(cfg_inst, ErrorStatus::None);
        }
    }

    #[test]
    fn deploy_error_status_retrying() {
        let instances = def_deps_w_error_status(ErrorStatus::Retrying);
        for cfg_inst in instances {
            match cfg_inst.target_status {
                TargetStatus::Deployed => validate_deploy_transition(cfg_inst, ErrorStatus::None),
                _ => validate_deploy_transition(cfg_inst, ErrorStatus::Retrying),
            }
        }
    }

    #[test]
    fn deploy_error_status_failed() {
        let instances = def_deps_w_error_status(ErrorStatus::Failed);
        for cfg_inst in instances {
            validate_deploy_transition(cfg_inst, ErrorStatus::Failed);
        }
    }

    fn validate_remove_transition(cfg_inst: ConfigInstance, expected_error_status: ErrorStatus) {
        let actual = fsm::remove(cfg_inst.clone());

        let expected = ConfigInstance {
            id: cfg_inst.id.clone(),
            target_status: cfg_inst.target_status,
            activity_status: ActivityStatus::Removed,
            error_status: expected_error_status,
            relative_filepath: cfg_inst.relative_filepath.clone(),
            patch_id: cfg_inst.patch_id.clone(),
            created_at: cfg_inst.created_at,
            updated_at: cfg_inst.updated_at,
            device_id: cfg_inst.device_id.clone(),
            config_schema_id: cfg_inst.config_schema_id.clone(),
            config_type_id: cfg_inst.config_type_id.clone(),
            attempts: 0,
            cooldown_ends_at: actual.cooldown_ends_at,
        };
        assert!(
            expected == actual,
            "expected:\n{expected:?}\n actual:\n{actual:?}\n",
        );

        // check the cooldown
        assert!(cfg_inst.cooldown_ends_at < Utc::now());
    }

    #[test]
    fn remove_error_status_none() {
        let instances = def_deps_w_error_status(ErrorStatus::None);
        for cfg_inst in instances {
            validate_remove_transition(cfg_inst, ErrorStatus::None);
        }
    }

    #[test]
    fn remove_error_status_retrying() {
        let instances = def_deps_w_error_status(ErrorStatus::Retrying);
        for cfg_inst in instances {
            match cfg_inst.target_status {
                TargetStatus::Removed | TargetStatus::Created => {
                    validate_remove_transition(cfg_inst, ErrorStatus::None)
                }
                _ => validate_remove_transition(cfg_inst, ErrorStatus::Retrying),
            }
        }
    }

    #[test]
    fn remove_error_status_failed() {
        let instances = def_deps_w_error_status(ErrorStatus::Failed);
        for cfg_inst in instances {
            validate_remove_transition(cfg_inst, ErrorStatus::Failed);
        }
    }

    fn validate_error_transition(
        cfg_inst: ConfigInstance,
        settings: &fsm::Settings,
        e: &impl MiruError,
        increment_attempts: bool,
    ) {
        let attempts = if increment_attempts && !e.is_network_connection_error() {
            cfg_inst.attempts + 1
        } else {
            cfg_inst.attempts
        };
        let expected_err_status =
            if attempts >= settings.max_attempts || cfg_inst.error_status == ErrorStatus::Failed {
                ErrorStatus::Failed
            } else {
                ErrorStatus::Retrying
            };
        let actual = fsm::error(cfg_inst.clone(), settings, e, increment_attempts);

        let expected = ConfigInstance {
            id: cfg_inst.id.clone(),
            target_status: cfg_inst.target_status,
            activity_status: cfg_inst.activity_status,
            error_status: expected_err_status,
            relative_filepath: cfg_inst.relative_filepath.clone(),
            patch_id: cfg_inst.patch_id.clone(),
            created_at: cfg_inst.created_at,
            updated_at: cfg_inst.updated_at,
            device_id: cfg_inst.device_id.clone(),
            config_schema_id: cfg_inst.config_schema_id.clone(),
            config_type_id: cfg_inst.config_type_id.clone(),
            attempts,
            cooldown_ends_at: actual.cooldown_ends_at,
        };
        assert!(
            expected == actual,
            "expected:\n{expected:?}\n actual:\n{actual:?}\n",
        );

        // check the cooldown
        let now = Utc::now();
        let cooldown = calc_exp_backoff(
            settings.exp_backoff_base_secs,
            2,
            attempts,
            settings.max_cooldown_secs,
        );
        let expected_cooldown_ends_at = now + TimeDelta::seconds(cooldown);
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
            for mut cfg_inst in instances {
                cfg_inst.attempts = 0;
                validate_error_transition(
                    cfg_inst.clone(),
                    &settings,
                    &MockMiruError::new(network_err),
                    increment_attempts,
                );
            }

            // failed attempts not reached max
            let instances = def_deps_w_all_status_combos();
            for mut cfg_inst in instances {
                cfg_inst.attempts = settings.max_attempts - 2;
                validate_error_transition(
                    cfg_inst.clone(),
                    &settings,
                    &MockMiruError::new(network_err),
                    increment_attempts,
                );
            }

            // failed attempts reached max
            let instances = def_deps_w_all_status_combos();
            for mut cfg_inst in instances {
                cfg_inst.attempts = settings.max_attempts - 1;
                validate_error_transition(
                    cfg_inst.clone(),
                    &settings,
                    &MockMiruError::new(network_err),
                    increment_attempts,
                );
            }

            // failed attempts exceeding max
            let instances = def_deps_w_all_status_combos();
            for mut cfg_inst in instances {
                cfg_inst.attempts = settings.max_attempts + 1;
                validate_error_transition(
                    cfg_inst.clone(),
                    &settings,
                    &MockMiruError::new(network_err),
                    increment_attempts,
                );
            }
        }
    }
}
