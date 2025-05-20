// internal crates
use config_agent_installer::errors::InstallerErr;
use config_agent_installer::users::{assert_groupname, assert_username};

// external crates
use users::{get_current_groupname, get_current_username};

pub mod assert_username {
    use super::*;

    #[tokio::test]
    async fn invalid() {
        let result = assert_username("invalid").unwrap_err();
        assert!(matches!(result, InstallerErr::InvalidOSUserErr { .. }));
    }

    #[tokio::test]
    async fn valid() {
        let username = get_current_username().unwrap();
        assert_username(&username.to_string_lossy()).unwrap();
    }
}

pub mod assert_groupname {
    use super::*;

    #[tokio::test]
    async fn invalid() {
        let result = assert_groupname("invalid").unwrap_err();
        assert!(matches!(result, InstallerErr::InvalidOSGroupErr { .. }));
    }

    #[tokio::test]
    async fn valid() {
        let groupname = get_current_groupname().unwrap();
        assert_groupname(&groupname.to_string_lossy()).unwrap();
    }
}
