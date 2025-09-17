// internal crates
use crate::errors::MiruError;
use crate::models::config_instance::{ActivityStatus, ConfigInstance, ErrorStatus, TargetStatus};
use crate::utils::calc_exp_backoff;

// external crates
use chrono::{TimeDelta, Utc};

// ================================ NEXT ACTION ==================================== //
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NextAction {
    None,
    Deploy,
    Remove,
    Archive,
    Wait(TimeDelta),
}

pub fn next_action(cfg_inst: &ConfigInstance, use_cooldown: bool) -> NextAction {
    // do nothing if the status is failed
    if cfg_inst.error_status == ErrorStatus::Failed {
        return NextAction::None;
    }

    // check for cooldown
    if use_cooldown && cfg_inst.is_in_cooldown() {
        return NextAction::Wait(cfg_inst.cooldown_ends_at.signed_duration_since(Utc::now()));
    }

    // determine the next action
    match cfg_inst.target_status {
        TargetStatus::Created | TargetStatus::Validated => match cfg_inst.activity_status {
            ActivityStatus::Created => NextAction::None,
            ActivityStatus::Validating => NextAction::None,
            ActivityStatus::Validated => NextAction::None,
            ActivityStatus::Queued => NextAction::Archive,
            ActivityStatus::Deployed => NextAction::Remove,
            ActivityStatus::Removed => NextAction::None,
        },
        TargetStatus::Deployed => match cfg_inst.activity_status {
            ActivityStatus::Created => NextAction::None,
            ActivityStatus::Validating => NextAction::None,
            ActivityStatus::Validated => NextAction::None,
            ActivityStatus::Queued => NextAction::Deploy,
            ActivityStatus::Deployed => NextAction::None,
            ActivityStatus::Removed => NextAction::Deploy,
        },
        TargetStatus::Removed => match cfg_inst.activity_status {
            ActivityStatus::Created => NextAction::Archive,
            ActivityStatus::Validating => NextAction::Archive,
            ActivityStatus::Validated => NextAction::Archive,
            ActivityStatus::Queued => NextAction::Archive,
            ActivityStatus::Deployed => NextAction::Remove,
            ActivityStatus::Removed => NextAction::None,
        },
    }
}

pub fn is_action_required(action: NextAction) -> bool {
    match action {
        NextAction::None => false,
        NextAction::Deploy => true,
        NextAction::Remove => true,
        NextAction::Archive => true,
        NextAction::Wait(_) => false,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Settings {
    pub max_attempts: u32,
    pub exp_backoff_base_secs: i64,
    pub max_cooldown_secs: i64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            max_attempts: 2147483647, // a VERY large number
            exp_backoff_base_secs: 15,
            max_cooldown_secs: 86400, // 24 hours
        }
    }
}

// ================================== TRANSITIONS ================================== //
#[derive(Debug)]
struct TransitionOptions {
    activity_status: Option<ActivityStatus>,
    error_status: Option<ErrorStatus>,
    attempts: Option<u32>,
    cooldown: Option<TimeDelta>,
}

fn transition(mut cfg_inst: ConfigInstance, options: TransitionOptions) -> ConfigInstance {
    if let Some(activity_status) = options.activity_status {
        cfg_inst.activity_status = activity_status;
    }

    if let Some(error_status) = options.error_status {
        cfg_inst.error_status = error_status;
    }

    if let Some(attempts) = options.attempts {
        cfg_inst.attempts = attempts;
    }

    if let Some(cooldown) = options.cooldown {
        cfg_inst.set_cooldown(cooldown);
    }

    cfg_inst
}

// ---------------------------- successful transitions ----------------------------= //
pub fn deploy(cfg_inst: ConfigInstance) -> ConfigInstance {
    let new_activity_status = ActivityStatus::Deployed;
    let options = get_success_options(&cfg_inst, new_activity_status);
    transition(cfg_inst, options)
}

pub fn remove(cfg_inst: ConfigInstance) -> ConfigInstance {
    let new_activity_status = ActivityStatus::Removed;
    let options = get_success_options(&cfg_inst, new_activity_status);
    transition(cfg_inst, options)
}

fn get_success_options(
    cfg_inst: &ConfigInstance,
    new_activity_status: ActivityStatus,
) -> TransitionOptions {
    TransitionOptions {
        activity_status: Some(new_activity_status),
        error_status: if has_recovered(cfg_inst, new_activity_status) {
            Some(ErrorStatus::None)
        } else {
            None
        },
        // reset attempts and cooldown
        attempts: if has_recovered(cfg_inst, new_activity_status) {
            Some(0)
        } else {
            None
        },
        cooldown: None,
    }
}

fn has_recovered(cfg_inst: &ConfigInstance, new_activity_status: ActivityStatus) -> bool {
    // the error status only needs to be updated if it is currently retrying. If is
    // failed then it can never exit failed and if it is None then it is already correct
    if cfg_inst.error_status != ErrorStatus::Retrying {
        return false;
    }

    // check if the new activity status matches the config instance's target status
    match cfg_inst.target_status {
        TargetStatus::Created | TargetStatus::Validated => {
            // the created and validated target statuses are a bit interesting in that
            // we're satisfied with the config instance being in other states as long as
            // it is not deployed.
            match new_activity_status {
                ActivityStatus::Created => true,
                ActivityStatus::Validating => true,
                ActivityStatus::Validated => true,
                ActivityStatus::Queued => true,
                ActivityStatus::Deployed => false,
                ActivityStatus::Removed => true,
            }
        }
        TargetStatus::Deployed => match new_activity_status {
            ActivityStatus::Created => false,
            ActivityStatus::Validating => false,
            ActivityStatus::Validated => false,
            ActivityStatus::Queued => false,
            ActivityStatus::Deployed => true,
            ActivityStatus::Removed => false,
        },
        TargetStatus::Removed => match new_activity_status {
            ActivityStatus::Created => false,
            ActivityStatus::Validating => false,
            ActivityStatus::Validated => false,
            ActivityStatus::Queued => false,
            ActivityStatus::Deployed => false,
            ActivityStatus::Removed => true,
        },
    }
}

// ----------------------------- error transitions --------------------------------- //
pub fn error(
    cfg_inst: ConfigInstance,
    settings: &Settings,
    e: &impl MiruError,
    increment_attempts: bool,
) -> ConfigInstance {
    let options = get_error_options(
        &cfg_inst,
        increment_attempts && should_increment_attempts(e),
        settings,
    );
    transition(cfg_inst, options)
}

fn should_increment_attempts(e: &impl MiruError) -> bool {
    !e.is_network_connection_error()
}

fn get_error_options(
    cfg_inst: &ConfigInstance,
    increment_attempts: bool,
    settings: &Settings,
) -> TransitionOptions {
    // determine the number of attempts
    let attempts = if increment_attempts {
        cfg_inst.attempts.saturating_add(1)
    } else {
        cfg_inst.attempts
    };

    // determine the new status
    let mut new_error_status = Some(ErrorStatus::Retrying);
    if attempts >= settings.max_attempts || cfg_inst.error_status == ErrorStatus::Failed {
        new_error_status = Some(ErrorStatus::Failed);
    }

    // determine the cooldown
    let cooldown = calc_exp_backoff(
        settings.exp_backoff_base_secs,
        2,
        attempts,
        settings.max_cooldown_secs,
    );

    TransitionOptions {
        activity_status: None,
        error_status: new_error_status,
        attempts: Some(attempts),
        cooldown: Some(TimeDelta::seconds(cooldown)),
    }
}
