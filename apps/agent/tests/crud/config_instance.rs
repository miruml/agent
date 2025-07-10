// internal crates
use config_agent::crud::config_instance::{
    matches_config_schema_and_activity_status, matches_filepath_and_activity_status,
};
use config_agent::models::config_instance::{ActivityStatus, ConfigInstance};

pub mod matches_config_schema_and_activity_status {
    use super::*;

    #[test]
    fn matches() {
        let config_schema_id = "123";
        let activity_status = ActivityStatus::Deployed;
        let cfg_inst = ConfigInstance {
            config_schema_id: config_schema_id.to_string(),
            activity_status,
            ..Default::default()
        };
        assert!(matches_config_schema_and_activity_status(
            &cfg_inst,
            config_schema_id,
            activity_status,
        ));
    }

    #[test]
    fn doesnt_match_config_schema_id() {
        let activity_status = ActivityStatus::Deployed;
        let cfg_inst = ConfigInstance {
            activity_status,
            ..Default::default()
        };
        assert!(!matches_config_schema_and_activity_status(
            &cfg_inst,
            "wrong_config_schema_id",
            activity_status,
        ));
    }

    #[test]
    pub fn doesnt_match_activity_status() {
        let config_schema_id = "123";
        let activity_status = ActivityStatus::Deployed;
        let cfg_inst = ConfigInstance {
            config_schema_id: config_schema_id.to_string(),
            activity_status,
            ..Default::default()
        };
        assert!(!matches_config_schema_and_activity_status(
            &cfg_inst,
            config_schema_id,
            ActivityStatus::Created,
        ));
    }
}

pub mod matches_filepath_and_activity_status {
    use super::*;

    #[test]
    fn matches() {
        let filepath = "test.txt";
        let activity_status = ActivityStatus::Deployed;
        let cfg_inst = ConfigInstance {
            relative_filepath: Some(filepath.to_string()),
            activity_status,
            ..Default::default()
        };
        assert!(matches_filepath_and_activity_status(
            &cfg_inst,
            filepath,
            activity_status
        ));
    }

    #[test]
    fn doesnt_match_filepath() {
        let activity_status = ActivityStatus::Deployed;
        let cfg_inst = ConfigInstance {
            activity_status,
            ..Default::default()
        };
        assert!(!matches_filepath_and_activity_status(
            &cfg_inst,
            "wrong_filepath",
            activity_status
        ));
    }

    #[test]
    pub fn doesnt_match_activity_status() {
        let filepath = "test.txt";
        let activity_status = ActivityStatus::Deployed;
        let cfg_inst = ConfigInstance {
            relative_filepath: Some(filepath.to_string()),
            activity_status,
            ..Default::default()
        };
        assert!(!matches_filepath_and_activity_status(
            &cfg_inst,
            filepath,
            ActivityStatus::Created
        ));
    }
}
