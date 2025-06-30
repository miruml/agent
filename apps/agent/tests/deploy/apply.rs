// std
use std::collections::HashMap;

// internal crates
use config_agent::cache::{
    entry::CacheEntry,
    file::FileCache,
};
use config_agent::deploy::{
    apply::{apply, find_instances_to_replace, find_replacement, is_dirty},
    errors::{DeployErr, ConflictingDeploymentsErr},
    fsm::Settings,
    fsm,
};
use config_agent::filesys::dir::Dir;
use config_agent::models::config_instance::{
    ConfigInstance,
    ActivityStatus,
    ErrorStatus,
    TargetStatus,
};
use config_agent::storage::config_instances::{
    ConfigInstanceCache,
    ConfigInstanceDataCache,
};

// external crates
use chrono::{Utc, TimeDelta};
use serde_json::json;

pub mod is_dirty_func {
    use super::*;

    #[tokio::test]
    async fn no_changes() {
        let instance = ConfigInstance {
            ..Default::default()
        };
        let new = &instance;
        let entry = CacheEntry {
            key: instance.id.clone(),
            value: instance.clone(),
            is_dirty: false,
            created_at: Utc::now(),
            last_accessed: Utc::now(),
        };
        let old = Some(&entry);
        assert!(!is_dirty(old, new));
    }

    #[tokio::test]
    async fn previous_is_none() {
        let instance = ConfigInstance {
            ..Default::default()
        };
        let new = &instance;
        assert!(is_dirty(None, new));
    }

    #[tokio::test]
    async fn previously_dirty() {
        let instance = ConfigInstance {
            ..Default::default()
        };
        let new = &instance;
        let entry = CacheEntry {
            key: instance.id.clone(),
            value: instance.clone(),
            is_dirty: true,
            created_at: Utc::now(),
            last_accessed: Utc::now(),
        };
        let old = Some(&entry);
        assert!(is_dirty(old, new));
    }


    #[tokio::test]
    async fn activity_status_changed() {
        let old = ConfigInstance {
            activity_status: ActivityStatus::Queued,
            ..Default::default()
        };
        let new = ConfigInstance {
            activity_status: ActivityStatus::Deployed,
            ..Default::default()
        };
        let entry = CacheEntry {
            key: old.id.clone(),
            value: old.clone(),
            is_dirty: false,
            created_at: Utc::now(),
            last_accessed: Utc::now(),
        };
        let old = Some(&entry);
        assert!(is_dirty(old, &new));
    }

    #[tokio::test]
    async fn error_status_changed() {
        let old = ConfigInstance {
            error_status: ErrorStatus::None,
            ..Default::default()
        };
        let new = ConfigInstance {
            error_status: ErrorStatus::Retrying,
            ..Default::default()
        };
        let entry = CacheEntry {
            key: old.id.clone(),
            value: old.clone(),
            is_dirty: false,
            created_at: Utc::now(),
            last_accessed: Utc::now(),
        };
        let old = Some(&entry);
        assert!(is_dirty(old, &new));
    }
}

pub mod apply_func {
    use super::*;

    #[tokio::test]
    async fn no_instances() {
        let dir = Dir::create_temp_dir("apply").await.unwrap();
        let (metadata_cache, _) = ConfigInstanceCache::spawn(
            dir.file("metadata.json"), 16,
        ).await.unwrap();
        let (instance_cache, _) = ConfigInstanceDataCache::spawn(
            dir.clone(), 16,
        ).await.unwrap();

        let result = apply(
            HashMap::new(),
            &metadata_cache,
            &instance_cache,
            &dir,
            &Settings::default(),
        ).await.unwrap();
        assert_eq!(result.len(), 0);
    }

    #[tokio::test]
    async fn deploy_1() {
        // define the instance 
        let instance = ConfigInstance {
            filepath: Some("/test/filepath".to_string()),
            // target status must be deployed to increment failure attempts
            target_status: TargetStatus::Deployed,
            ..Default::default()
        };

        // create the cache but omit the instance data
        let dir = Dir::create_temp_dir("deploy").await.unwrap();
        let (metadata_cache, _) = ConfigInstanceCache::spawn(
            dir.file("metadata.json"), 16,
        ).await.unwrap();
        let (instance_cache, _) = ConfigInstanceDataCache::spawn(
            dir.clone(), 16,
        ).await.unwrap();
        instance_cache.write(
            instance.id.clone(), json!({"speed": 4}), |_, _| false, true,
        ).await.unwrap();

        // deploy the instance
        let settings = Settings::default();
        let instances_to_apply = HashMap::from([(instance.id.clone(), instance.clone())]);
        let result = apply(
            instances_to_apply,
            &metadata_cache,
            &instance_cache,
            &dir,
            &settings,
        ).await.unwrap();
        assert_eq!(result.len(), 1);
        let actual = result[&instance.id].clone();

        // define the expected instance
        let expected = ConfigInstance {
            activity_status: ActivityStatus::Deployed,
            ..instance
        };

        // check that the returned instances' states were correctly updated
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn deploy_1_failure_1_success() {
        // define the instances 
        let instance1 = ConfigInstance {
            filepath: Some("/test/filepath1".to_string()),
            // target status must be deployed to increment failure attempts
            target_status: TargetStatus::Deployed,
            ..Default::default()
        };
        let instance2 = ConfigInstance {
            filepath: Some("/test/filepath2".to_string()),
            target_status: TargetStatus::Deployed,
            ..Default::default()
        };

        // create the cache but omit the instance data
        let dir = Dir::create_temp_dir("deploy").await.unwrap();
        let (metadata_cache, _) = ConfigInstanceCache::spawn(
            dir.file("metadata.json"), 16,
        ).await.unwrap();
        let (instance_cache, _) = ConfigInstanceDataCache::spawn(
            dir.clone(), 16,
        ).await.unwrap();
        instance_cache.write(
            instance1.id.clone(), json!({"speed": 4}), |_, _| false, true,
        ).await.unwrap();

        // deploy the instance
        let settings = Settings::default();
        let instances_to_apply = HashMap::from([
            (instance1.id.clone(), instance1.clone()),
            (instance2.id.clone(), instance2.clone()),
        ]);
        let result = apply(
            instances_to_apply,
            &metadata_cache,
            &instance_cache,
            &dir,
            &settings,
        ).await.unwrap();
        assert_eq!(result.len(), 2);
        let actual1 = result[&instance1.id].clone();
        let actual2 = result[&instance2.id].clone();

        // define the expected instances
        let expected1 = ConfigInstance {
            activity_status: ActivityStatus::Deployed,
            ..instance1
        };
        let expected2 = ConfigInstance {
            activity_status: ActivityStatus::Removed,
            error_status: ErrorStatus::Retrying,
            attempts: 1,
            cooldown_ends_at: actual2.cooldown_ends_at,
            ..instance2
        };
        let cooldown = fsm::calc_exp_backoff(
            2,
            settings.exp_backoff_base_secs,
            expected2.attempts,
            settings.max_cooldown_secs,
        );
        let approx_cooldown_ends_at = Utc::now() + TimeDelta::seconds(cooldown as i64);
        assert!(expected2.cooldown_ends_at <= approx_cooldown_ends_at);
        assert!(expected2.cooldown_ends_at >= approx_cooldown_ends_at - TimeDelta::seconds(1));

        // check that the returned instances' states were correctly updated
        assert_eq!(expected1, actual1);
        assert_eq!(expected2, actual2);
    }

    #[tokio::test]
    async fn remove_1() {
        // define the instance 
        let filepath = "/test/filepath".to_string();
        let instance = ConfigInstance {
            filepath: Some(filepath.clone()),
            target_status: TargetStatus::Removed,
            ..Default::default()
        };

        // create the instance in the cache
        let dir = Dir::create_temp_dir("deploy").await.unwrap();
        let (metadata_cache, _) = ConfigInstanceCache::spawn(
            dir.file("metadata.json"), 16,
        ).await.unwrap();
        let (instance_cache, _) = ConfigInstanceDataCache::spawn(
            dir.clone(), 16,
        ).await.unwrap();
        instance_cache.write(
            instance.id.clone(), json!({"speed": 4}), |_, _| false, true,
        ).await.unwrap();

        // deploy the instance
        let settings = Settings::default();
        let instances_to_apply = HashMap::from([(instance.id.clone(), instance.clone())]);
        let result = apply(
            instances_to_apply,
            &metadata_cache,
            &instance_cache,
            &dir,
            &settings,
        ).await.unwrap();
        assert_eq!(result.len(), 1);
        let actual = result[&instance.id].clone();

        // define the expected instance
        let expected = ConfigInstance {
            activity_status: ActivityStatus::Removed,
            ..instance
        };

        // check that the returned instances' states were correctly updated
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn rollback_1_deploy_failed_different_config_schemas() {
        // define the instances with DIFFERENT config schemas but the same filepath
        let filepath = "/test/filepath".to_string();
        let to_deploy= ConfigInstance {
            filepath: Some(filepath.clone()),
            target_status: TargetStatus::Deployed,
            activity_status: ActivityStatus::Queued,
            ..Default::default()
        };
        let to_remove = ConfigInstance {
            filepath: Some(filepath.clone()),
            target_status: TargetStatus::Removed,
            activity_status: ActivityStatus::Deployed,
            ..Default::default()
        };

        // create the cache but omit the instance data
        let dir = Dir::create_temp_dir("deploy").await.unwrap();
        let (metadata_cache, _) = ConfigInstanceCache::spawn(
            dir.file("metadata.json"), 16,
        ).await.unwrap();
        metadata_cache.write(
            to_remove.id.clone(), to_remove.clone(), |_, _| false, true,
        ).await.unwrap();
        metadata_cache.write(
            to_deploy.id.clone(), to_deploy.clone(), |_, _| false, true,
        ).await.unwrap();
        let (instance_cache, _) = ConfigInstanceDataCache::spawn(
            dir.clone(), 16,
        ).await.unwrap();
        let to_remove_data = json!({"speed": 4});
        instance_cache.write(
            to_remove.id.clone(), to_remove_data.clone(), |_, _| false, true,
        ).await.unwrap();

        // deploy the instance
        let settings = Settings::default();
        let instances_to_apply = HashMap::from([
            (to_deploy.id.clone(), to_deploy.clone()),
            (to_remove.id.clone(), to_remove.clone()),
        ]);
        let result = apply(
            instances_to_apply,
            &metadata_cache,
            &instance_cache,
            &dir,
            &settings,
        ).await.unwrap();
        assert_eq!(result.len(), 2);
        let actual_to_remove = result[&to_remove.id].clone();
        let actual_to_deploy = result[&to_deploy.id].clone();

        // define the expected instances
        let expected_to_remove = ConfigInstance {
            activity_status: ActivityStatus::Removed,
            ..to_remove
        };

        let expected_to_deploy = ConfigInstance {
            activity_status: ActivityStatus::Removed,
            error_status: ErrorStatus::Retrying,
            attempts: 1,
            cooldown_ends_at: actual_to_deploy.cooldown_ends_at,
            ..to_deploy
        };
        let cooldown = fsm::calc_exp_backoff(
            2,
            settings.exp_backoff_base_secs,
            expected_to_deploy.attempts,
            settings.max_cooldown_secs,
        );
        let approx_cooldown_ends_at = Utc::now() + TimeDelta::seconds(cooldown as i64);
        assert!(expected_to_deploy.cooldown_ends_at <= approx_cooldown_ends_at);
        assert!(expected_to_deploy.cooldown_ends_at >= approx_cooldown_ends_at - TimeDelta::seconds(1));

        // check that the returned instances' states were correctly updated
        assert_eq!(expected_to_remove, actual_to_remove);
        assert_eq!(expected_to_deploy, actual_to_deploy);
    }

    #[tokio::test]
    async fn rollback_1_deploy_failed_same_config_schema() {
        // define the instances with DIFFERENT config schemas but the same filepath
        let filepath = "/test/filepath".to_string();
        let to_deploy= ConfigInstance {
            filepath: Some(filepath.clone()),
            target_status: TargetStatus::Deployed,
            activity_status: ActivityStatus::Queued,
            ..Default::default()
        };
        let to_remove = ConfigInstance {
            config_schema_id: to_deploy.config_schema_id.clone(),
            filepath: Some(filepath.clone()),
            target_status: TargetStatus::Removed,
            activity_status: ActivityStatus::Deployed,
            ..Default::default()
        };

        // create the cache but omit the instance data
        let dir = Dir::create_temp_dir("deploy").await.unwrap();
        let (metadata_cache, _) = ConfigInstanceCache::spawn(
            dir.file("metadata.json"), 16,
        ).await.unwrap();
        metadata_cache.write(
            to_remove.id.clone(), to_remove.clone(), |_, _| false, true,
        ).await.unwrap();
        metadata_cache.write(
            to_deploy.id.clone(), to_deploy.clone(), |_, _| false, true,
        ).await.unwrap();
        let (instance_cache, _) = ConfigInstanceDataCache::spawn(
            dir.clone(), 16,
        ).await.unwrap();
        let to_remove_data = json!({"speed": 4});
        instance_cache.write(
            to_remove.id.clone(), to_remove_data.clone(), |_, _| false, true,
        ).await.unwrap();

        // deploy the instance
        let settings = Settings::default();
        let instances_to_apply = HashMap::from([
            (to_deploy.id.clone(), to_deploy.clone()),
            (to_remove.id.clone(), to_remove.clone()),
        ]);
        let result = apply(
            instances_to_apply,
            &metadata_cache,
            &instance_cache,
            &dir,
            &settings,
        ).await.unwrap();
        assert_eq!(result.len(), 2);
        let actual_to_remove = result[&to_remove.id].clone();
        let actual_to_deploy = result[&to_deploy.id].clone();

        // define the expected instances
        let expected_to_deploy = ConfigInstance {
            activity_status: ActivityStatus::Removed,
            error_status: ErrorStatus::Retrying,
            attempts: 1,
            cooldown_ends_at: actual_to_deploy.cooldown_ends_at,
            ..to_deploy
        };
        let cooldown = fsm::calc_exp_backoff(
            2,
            settings.exp_backoff_base_secs,
            expected_to_deploy.attempts,
            settings.max_cooldown_secs,
        );
        let approx_cooldown_ends_at = Utc::now() + TimeDelta::seconds(cooldown as i64);
        assert!(expected_to_deploy.cooldown_ends_at <= approx_cooldown_ends_at);
        assert!(expected_to_deploy.cooldown_ends_at >= approx_cooldown_ends_at - TimeDelta::seconds(1));

        let expected_to_remove = ConfigInstance {
            activity_status: ActivityStatus::Deployed,
            cooldown_ends_at: actual_to_remove.cooldown_ends_at,
            ..to_remove
        };

        // check that the returned instances' states were correctly updated
        assert_eq!(expected_to_remove, actual_to_remove);
        assert_eq!(expected_to_deploy, actual_to_deploy);
    }

    #[tokio::test]
    async fn replace_1() {
        // define the instances with DIFFERENT config schemas but the same filepath
        let filepath = "/test/filepath".to_string();
        let to_deploy= ConfigInstance {
            filepath: Some(filepath.clone()),
            target_status: TargetStatus::Deployed,
            activity_status: ActivityStatus::Queued,
            ..Default::default()
        };
        let to_remove = ConfigInstance {
            config_schema_id: to_deploy.config_schema_id.clone(),
            filepath: Some(filepath.clone()),
            target_status: TargetStatus::Removed,
            activity_status: ActivityStatus::Deployed,
            ..Default::default()
        };

        // create the cache but omit the instance data
        let dir = Dir::create_temp_dir("deploy").await.unwrap();
        let (metadata_cache, _) = ConfigInstanceCache::spawn(
            dir.file("metadata.json"), 16,
        ).await.unwrap();
        metadata_cache.write(
            to_remove.id.clone(), to_remove.clone(), |_, _| false, true,
        ).await.unwrap();
        metadata_cache.write(
            to_deploy.id.clone(), to_deploy.clone(), |_, _| false, true,
        ).await.unwrap();
        let (instance_cache, _) = ConfigInstanceDataCache::spawn(
            dir.clone(), 16,
        ).await.unwrap();
        let to_remove_data = json!({"speed": 4});
        instance_cache.write(
            to_remove.id.clone(), to_remove_data.clone(), |_, _| false, true,
        ).await.unwrap();
        let to_deploy_data = json!({"speed": 5});
        instance_cache.write(
            to_deploy.id.clone(), to_deploy_data.clone(), |_, _| false, true,
        ).await.unwrap();

        // deploy the instance
        let settings = Settings::default();
        let instances_to_apply = HashMap::from([
            (to_deploy.id.clone(), to_deploy.clone()),
            (to_remove.id.clone(), to_remove.clone()),
        ]);
        let result = apply(
            instances_to_apply,
            &metadata_cache,
            &instance_cache,
            &dir,
            &settings,
        ).await.unwrap();
        assert_eq!(result.len(), 2);
        let actual_to_remove = result[&to_remove.id].clone();
        let actual_to_deploy = result[&to_deploy.id].clone();

        // define the expected instances
        let expected_to_remove = ConfigInstance {
            activity_status: ActivityStatus::Removed,
            ..to_remove
        };
        let expected_to_deploy = ConfigInstance {
            activity_status: ActivityStatus::Deployed,
            ..to_deploy
        };

        // check that the returned instances' states were correctly updated
        assert_eq!(expected_to_remove, actual_to_remove);
        assert_eq!(expected_to_deploy, actual_to_deploy);
    }
}

pub mod find_instances_to_replace_func {
    use super::*;

    #[tokio::test]
    async fn no_matches() {
        let dir = Dir::create_temp_dir("apply").await.unwrap();
        let (cache, _) = FileCache::spawn(
            dir.file("cache.json"), 16,
        ).await.unwrap();

        let instance = ConfigInstance {
            filepath: Some("/test/filepath".to_string()),
            ..Default::default()
        };

        // create a bunch of instances with that don't match
        for i in 0..10 {
            let instance = ConfigInstance {
                filepath: Some(format!("/test/filepath{}", i)),
                activity_status: ActivityStatus::Deployed,
                target_status: TargetStatus::Removed,
                ..Default::default()
            };
            cache.write(instance.id.clone(), instance, |_, _| false, true,
            ).await.unwrap();
        }
        let result = find_instances_to_replace(
            &instance, &cache,
        ).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn one_file_path_match() {
        let dir = Dir::create_temp_dir("apply").await.unwrap();
        let (cache, _) = FileCache::spawn(
            dir.file("cache.json"), 16,
        ).await.unwrap();

        let filepath = "/test/filepath".to_string();
        let instance = ConfigInstance {
            filepath: Some(filepath.clone()),
            ..Default::default()
        };

        // create a valid replacement instance
        let to_replace = ConfigInstance {
            filepath: Some(filepath.clone()),
            activity_status: ActivityStatus::Deployed,
            target_status: TargetStatus::Removed,
            ..Default::default()
        };
        cache.write(
            to_replace.id.clone(), to_replace.clone(), |_, _| false, true,
        ).await.unwrap();

        let result = find_instances_to_replace(
            &instance, &cache,
        ).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], to_replace);
    }

    #[tokio::test]
    async fn one_config_schema_match() {
        let dir = Dir::create_temp_dir("apply").await.unwrap();
        let (cache, _) = FileCache::spawn(
            dir.file("cache.json"), 16,
        ).await.unwrap();

        let instance = ConfigInstance {
            ..Default::default()
        };

        // create a valid replacement instance
        let to_replace = ConfigInstance {
            config_schema_id: instance.config_schema_id.clone(),
            activity_status: ActivityStatus::Deployed,
            target_status: TargetStatus::Removed,
            ..Default::default()
        };
        cache.write(
            to_replace.id.clone(), to_replace.clone(), |_, _| false, true,
        ).await.unwrap();

        let result = find_instances_to_replace(
            &instance, &cache,
        ).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], to_replace);
    }

    #[tokio::test]
    async fn multiple_matches() {
        let dir = Dir::create_temp_dir("apply").await.unwrap();
        let (cache, _) = FileCache::spawn(
            dir.file("cache.json"), 16,
        ).await.unwrap();

        let instance = ConfigInstance {
            filepath: Some("/test/filepath".to_string()),
            ..Default::default()
        };

        // create a valid replacement instance
        let filepath_to_replace = ConfigInstance {
            filepath: instance.filepath.clone(),
            activity_status: ActivityStatus::Deployed,
            target_status: TargetStatus::Removed,
            ..Default::default()
        };
        cache.write(
            filepath_to_replace.id.clone(), filepath_to_replace.clone(), |_, _| false, true,
        ).await.unwrap();
        let schema_to_replace = ConfigInstance {
            config_schema_id: instance.config_schema_id.clone(),
            activity_status: ActivityStatus::Deployed,
            target_status: TargetStatus::Removed,
            ..Default::default()
        };
        cache.write(
            schema_to_replace.id.clone(), schema_to_replace.clone(), |_, _| false, true,
        ).await.unwrap();

        let result = find_instances_to_replace(
            &instance, &cache,
        ).await.unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.contains(&schema_to_replace));
        assert!(result.contains(&filepath_to_replace));
    }

    #[tokio::test]
    async fn conflicting_target_status() {
        let dir = Dir::create_temp_dir("apply").await.unwrap();
        let (cache, _) = FileCache::spawn(
            dir.file("cache.json"), 16,
        ).await.unwrap();

        let instance = ConfigInstance {
            ..Default::default()
        };

        // create a valid replacement instance
        let to_replace = ConfigInstance {
            config_schema_id: instance.config_schema_id.clone(),
            activity_status: ActivityStatus::Deployed,
            target_status: TargetStatus::Deployed,
            ..Default::default()
        };
        cache.write(
            to_replace.id.clone(), to_replace.clone(), |_, _| false, true,
        ).await.unwrap();

        let error= find_instances_to_replace(
            &instance, &cache,
        ).await.unwrap_err();
        assert!(matches!(error, DeployErr::ConflictingDeploymentsErr(
            ConflictingDeploymentsErr { .. }
        )));
    }

}

pub mod find_replacement_func {
    use super::*;

    #[tokio::test]
    async fn no_matches() {
        let dir = Dir::create_temp_dir("apply").await.unwrap();
        let (cache, _) = FileCache::spawn(
            dir.file("cache.json"), 16,
        ).await.unwrap();

        let instance = ConfigInstance {
            ..Default::default()
        };

        // create a bunch of instances with that don't match
        for _ in 0..10 {
            let instance = ConfigInstance {
                activity_status: ActivityStatus::Queued,
                target_status: TargetStatus::Deployed,
                ..Default::default()
            };
            cache.write(instance.id.clone(), instance, |_, _| false, true,
            ).await.unwrap();
        }

        let result = find_replacement(
            &instance, &cache,
        ).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn one_match_no_cooldown() {
        let dir = Dir::create_temp_dir("apply").await.unwrap();
        let (cache, _) = FileCache::spawn(
            dir.file("cache.json"), 16,
        ).await.unwrap();

        let instance = ConfigInstance {
            ..Default::default()
        };

        // create a valid replacement instance
        let replacement = ConfigInstance {
            config_schema_id: instance.config_schema_id.clone(),
            activity_status: ActivityStatus::Queued,
            target_status: TargetStatus::Deployed,
            ..Default::default()
        };
        cache.write(replacement.id.clone(), replacement.clone(), |_, _| false, true,
        ).await.unwrap();

        // find the replacement
        let result = find_replacement(
            &instance, &cache,
        ).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), replacement);
    }

    #[tokio::test]
    async fn one_match_in_cooldown() {
        let dir = Dir::create_temp_dir("apply").await.unwrap();
        let (cache, _) = FileCache::spawn(
            dir.file("cache.json"), 16,
        ).await.unwrap();

        let instance = ConfigInstance {
            ..Default::default()
        };

        // create a valid replacement instance
        let replacement = ConfigInstance {
            config_schema_id: instance.config_schema_id.clone(),
            activity_status: ActivityStatus::Queued,
            target_status: TargetStatus::Deployed,
            cooldown_ends_at: Utc::now() + TimeDelta::seconds(10),
            ..Default::default()
        };
        cache.write(replacement.id.clone(), replacement.clone(), |_, _| false, true,
        ).await.unwrap();

        // find the replacement
        let result = find_replacement(
            &instance, &cache,
        ).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), replacement);
    }

    #[tokio::test]
    async fn multiple_matches_no_cooldown() {
        let dir = Dir::create_temp_dir("apply").await.unwrap();
        let (cache, _) = FileCache::spawn(
            dir.file("cache.json"), 16,
        ).await.unwrap();

        let instance = ConfigInstance {
            ..Default::default()
        };

        // create a valid replacement instance
        let replacement1 = ConfigInstance {
            config_schema_id: instance.config_schema_id.clone(),
            activity_status: ActivityStatus::Queued,
            target_status: TargetStatus::Deployed,
            cooldown_ends_at: Utc::now() + TimeDelta::seconds(10),
            ..Default::default()
        };
        cache.write(replacement1.id.clone(), replacement1.clone(), |_, _| false, true,
        ).await.unwrap();
        let replacement2 = ConfigInstance {
            config_schema_id: instance.config_schema_id.clone(),
            activity_status: ActivityStatus::Queued,
            target_status: TargetStatus::Deployed,
            cooldown_ends_at: Utc::now() + TimeDelta::seconds(10),
            ..Default::default()
        };
        cache.write(replacement2.id.clone(), replacement2.clone(), |_, _| false, true,
        ).await.unwrap();

        // find the replacement
        find_replacement(&instance, &cache).await.unwrap_err();
    }
}
