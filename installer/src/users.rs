use crate::errors::{
    InstallerErr,
    InvalidOSUserErr,
    InvalidOSGroupErr,
};
use users::{get_current_username, get_current_groupname};
use config_agent::trace;

pub fn assert_username(target_username: &str) -> Result<(), InstallerErr> {
    let username = match get_current_username() {
        Some(username) => username,
        None => {
            return Err(InstallerErr::InvalidOSUserErr(InvalidOSUserErr {
                target_user: target_username.to_string(),
                actual_user: "unknown".to_string(),
                trace: trace!(),
            }));
        }
    };
    if username != target_username {
        return Err(InstallerErr::InvalidOSUserErr(InvalidOSUserErr {
            target_user: target_username.to_string(),
            actual_user: username.to_string_lossy().to_string(),
            trace: trace!(),
        }));
    }
    Ok(())
}

pub fn assert_groupname(target_groupname: &str) -> Result<(), InstallerErr> {
    let groupname = match get_current_groupname() {
        Some(groupname) => groupname,
        None => {
            return Err(InstallerErr::InvalidOSGroupErr(InvalidOSGroupErr {
                target_group: target_groupname.to_string(),
                actual_group: "unknown".to_string(),
                trace: trace!(),
            }));
        }
    };
    if groupname != target_groupname {
        return Err(InstallerErr::InvalidOSGroupErr(InvalidOSGroupErr {
            target_group: target_groupname.to_string(),
            actual_group: groupname.to_string_lossy().to_string(),
            trace: trace!(),
        }));
    }
    Ok(())
}