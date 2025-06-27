// standard crates
use std::cmp::min;

// internal crates
use crate::errors::MiruError;
use crate::models::config_instance::{
    ConfigInstance, ConfigInstanceActivityStatus, ConfigInstanceErrorStatus,
    ConfigInstanceTargetStatus,
};

// external crates
use chrono::{TimeDelta, Utc};

// ================================ NEXT ACTION ==================================== //
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NextAction {
    None,
    Deploy,
    Remove,
    Wait(TimeDelta),
}

pub fn next_action(instance: &ConfigInstance, use_cooldown: bool) -> NextAction {
    // check for cooldown
    if use_cooldown && instance.is_cooling_down() {
        return NextAction::Wait(instance.cooldown_ends_at.signed_duration_since(Utc::now()));
    }

    // do nothing if the status is failed
    if instance.error_status == ConfigInstanceErrorStatus::Failed {
        return NextAction::None;
    }

    // determine the next action
    match instance.target_status {
        ConfigInstanceTargetStatus::Created => match instance.activity_status {
            ConfigInstanceActivityStatus::Created => NextAction::None,
            ConfigInstanceActivityStatus::Queued => NextAction::None,
            ConfigInstanceActivityStatus::Deployed => NextAction::Remove,
            ConfigInstanceActivityStatus::Removed => NextAction::None,
        },
        ConfigInstanceTargetStatus::Deployed => match instance.activity_status {
            ConfigInstanceActivityStatus::Created => NextAction::Deploy,
            ConfigInstanceActivityStatus::Queued => NextAction::Deploy,
            ConfigInstanceActivityStatus::Deployed => NextAction::None,
            ConfigInstanceActivityStatus::Removed => NextAction::Deploy,
        },
        ConfigInstanceTargetStatus::Removed => match instance.activity_status {
            ConfigInstanceActivityStatus::Created => NextAction::Remove,
            ConfigInstanceActivityStatus::Queued => NextAction::Remove,
            ConfigInstanceActivityStatus::Deployed => NextAction::Remove,
            ConfigInstanceActivityStatus::Removed => NextAction::None,
        },
    }
}

pub fn is_action_required(cfg_inst: &ConfigInstance) -> bool {
    match next_action(cfg_inst, true) {
        NextAction::None => false,
        NextAction::Deploy => true,
        NextAction::Remove => true,
        NextAction::Wait(_) => false,
    }
}

pub struct Settings {
    max_attempts: u32,
    exp_backoff_base_secs: u32,
    max_cooldown_secs: u32,
}

// ================================== TRANSITIONS ================================== //
#[derive(Debug)]
struct TransitionOptions {
    activity_status: Option<ConfigInstanceActivityStatus>,
    error_status: Option<ConfigInstanceErrorStatus>,
    attempts: Option<u32>,
    cooldown: Option<TimeDelta>,
}

fn transition(mut instance: ConfigInstance, options: TransitionOptions) -> ConfigInstance {
    if let Some(activity_status) = options.activity_status {
        instance.activity_status = activity_status;
    }

    if let Some(error_status) = options.error_status {
        instance.error_status = error_status;
    }

    if let Some(attempts) = options.attempts {
        instance.attempts = attempts;
    }

    if let Some(cooldown) = options.cooldown {
        instance.set_cooldown(cooldown);
    }

    instance
}

// ---------------------------- successful transitions ----------------------------= //
pub fn deploy(instance: ConfigInstance) -> ConfigInstance {
    let new_activity_status = ConfigInstanceActivityStatus::Deployed;
    transition(instance, get_success_options(new_activity_status))
}

pub fn remove(instance: ConfigInstance) -> ConfigInstance {
    let new_activity_status = ConfigInstanceActivityStatus::Removed;
    transition(instance, get_success_options(new_activity_status))
}

fn get_success_options(new_activity_status: ConfigInstanceActivityStatus) -> TransitionOptions {
    TransitionOptions {
        activity_status: Some(new_activity_status),
        error_status: None,
        // reset attempts and cooldown
        attempts: Some(0),
        cooldown: Some(TimeDelta::zero()),
    }
}

// ----------------------------- error transitions --------------------------------- //
pub fn error(instance: ConfigInstance, settings: &Settings, e: &impl MiruError) -> ConfigInstance {
    let options = get_error_options(
        &instance, should_increment_attempts(e), settings,
    );
    transition(instance, options)
}

fn should_increment_attempts(e: &impl MiruError) -> bool {
    !e.is_network_connection_error()
}

fn get_error_options(
    instance: &ConfigInstance,
    increment_attempts: bool,
    settings: &Settings,
) -> TransitionOptions {
    // determine the number of attempts
    let attempts = if increment_attempts {
        instance.attempts.saturating_add(1)
    } else {
        instance.attempts
    };

    // determine the new status
    let mut new_error_status = Some(ConfigInstanceErrorStatus::Retrying);
    if attempts >= settings.max_attempts {
        new_error_status = Some(ConfigInstanceErrorStatus::Failed);
    }

    // determine the cooldown
    let cooldown = calc_exp_backoff(
        settings.exp_backoff_base_secs,
        attempts,
        settings.max_cooldown_secs,
    );

    TransitionOptions {
        activity_status: None,
        error_status: new_error_status,
        attempts: Some(attempts),
        cooldown: Some(TimeDelta::seconds(cooldown as i64)),
    }
}

pub fn calc_exp_backoff(base: u32, exp: u32, max: u32) -> u32 {
    min(2u32.saturating_pow(exp).saturating_mul(base), max)
}
