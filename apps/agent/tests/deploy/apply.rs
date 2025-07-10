// std
use std::collections::HashMap;

// internal crates
use config_agent::cache::{entry::CacheEntry, file::FileCache};
use config_agent::deploy::{
    apply::{apply, find_instances_to_replace, find_replacement, is_dirty},
    errors::DeployErr,
    fsm::Settings,
};
use config_agent::filesys::dir::Dir;
use config_agent::models::config_instance::{
    ActivityStatus, ConfigInstance, ErrorStatus, TargetStatus,
};
use config_agent::storage::config_instances::{ConfigInstanceCache, ConfigInstanceContentCache};
use config_agent::utils::calc_exp_backoff;

// external crates
use chrono::{TimeDelta, Utc};
use serde_json::json;

pub mod is_dirty_func {
    use super::*;

    #[tokio::test]
    async fn no_changes() {
        let cfg_inst = ConfigInstance {
            ..Default::default()
        };
        let new = &cfg_inst;
        let entry = CacheEntry {
            key: cfg_inst.id.clone(),
            value: cfg_inst.clone(),
            is_dirty: false,
            created_at: Utc::now(),
            last_accessed: Utc::now(),
        };
        let old = Some(&entry);
        assert!(!is_dirty(old, new));
    }

    #[tokio::test]
    async fn previous_is_none() {
        let cfg_inst = ConfigInstance {
            ..Default::default()
        };
        let new = &cfg_inst;
        assert!(is_dirty(None, new));
    }

    #[tokio::test]
    async fn previously_dirty() {
        let cfg_inst = ConfigInstance {
            ..Default::default()
        };
        let new = &cfg_inst;
        let entry = CacheEntry {
            key: cfg_inst.id.clone(),
            value: cfg_inst.clone(),
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
        let (cfg_inst_cache, _) = ConfigInstanceCache::spawn(16, dir.file("metadata.json"), 1000)
            .await
            .unwrap();
        let (cfg_inst_content_cache, _) = ConfigInstanceContentCache::spawn(16, dir.clone(), 1000)
            .await
            .unwrap();

        let result = apply(
            HashMap::new(),
            &cfg_inst_cache,
            &cfg_inst_content_cache,
            &dir,
            &Settings::default(),
        )
        .await
        .unwrap();
        assert_eq!(result.len(), 0);
    }

    #[tokio::test]
    async fn deploy_1() {
        // define the config instance
        let cfg_inst = ConfigInstance {
            relative_filepath: Some("/test/filepath".to_string()),
            // target status must be deployed to increment failure attempts
            target_status: TargetStatus::Deployed,
            ..Default::default()
        };

        // create the cache but omit the config instance content
        let dir = Dir::create_temp_dir("deploy").await.unwrap();
        let (cfg_inst_cache, _) = ConfigInstanceCache::spawn(16, dir.file("metadata.json"), 1000)
            .await
            .unwrap();
        let (cfg_inst_content_cache, _) = ConfigInstanceContentCache::spawn(16, dir.clone(), 1000)
            .await
            .unwrap();
        cfg_inst_content_cache
            .write(cfg_inst.id.clone(), json!({"speed": 4}), |_, _| false, true)
            .await
            .unwrap();

        // deploy the config instance
        let settings = Settings::default();
        let cfg_insts_to_apply = HashMap::from([(cfg_inst.id.clone(), cfg_inst.clone())]);
        let result = apply(
            cfg_insts_to_apply,
            &cfg_inst_cache,
            &cfg_inst_content_cache,
            &dir,
            &settings,
        )
        .await
        .unwrap();
        assert_eq!(result.len(), 1);
        let actual = result[&cfg_inst.id].clone();

        // define the expected config instance
        let expected = ConfigInstance {
            activity_status: ActivityStatus::Deployed,
            ..cfg_inst
        };

        // check that the returned config instances' states were correctly updated
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn deploy_1_failure_1_success() {
        // define the config instances
        let cfg_inst1 = ConfigInstance {
            relative_filepath: Some("/test/filepath1".to_string()),
            // target status must be deployed to increment failure attempts
            target_status: TargetStatus::Deployed,
            ..Default::default()
        };
        let cfg_inst2 = ConfigInstance {
            relative_filepath: Some("/test/filepath2".to_string()),
            target_status: TargetStatus::Deployed,
            ..Default::default()
        };

        // create the cache but omit the config instance content
        let dir = Dir::create_temp_dir("deploy").await.unwrap();
        let (cfg_inst_cache, _) = ConfigInstanceCache::spawn(16, dir.file("metadata.json"), 1000)
            .await
            .unwrap();
        let (cfg_inst_content_cache, _) = ConfigInstanceContentCache::spawn(16, dir.clone(), 1000)
            .await
            .unwrap();
        cfg_inst_content_cache
            .write(
                cfg_inst1.id.clone(),
                json!({"speed": 4}),
                |_, _| false,
                true,
            )
            .await
            .unwrap();

        // deploy the config instance
        let settings = Settings::default();
        let cfg_insts_to_apply = HashMap::from([
            (cfg_inst1.id.clone(), cfg_inst1.clone()),
            (cfg_inst2.id.clone(), cfg_inst2.clone()),
        ]);
        let result = apply(
            cfg_insts_to_apply,
            &cfg_inst_cache,
            &cfg_inst_content_cache,
            &dir,
            &settings,
        )
        .await
        .unwrap();
        assert_eq!(result.len(), 2);
        let actual1 = result[&cfg_inst1.id].clone();
        let actual2 = result[&cfg_inst2.id].clone();

        // define the expected config instances
        let expected1 = ConfigInstance {
            activity_status: ActivityStatus::Deployed,
            ..cfg_inst1
        };
        let expected2 = ConfigInstance {
            activity_status: ActivityStatus::Removed,
            error_status: ErrorStatus::Retrying,
            attempts: 1,
            cooldown_ends_at: actual2.cooldown_ends_at,
            ..cfg_inst2
        };
        let cooldown = calc_exp_backoff(
            settings.exp_backoff_base_secs,
            2,
            expected2.attempts,
            settings.max_cooldown_secs,
        );
        let approx_cooldown_ends_at = Utc::now() + TimeDelta::seconds(cooldown as i64);
        assert!(expected2.cooldown_ends_at <= approx_cooldown_ends_at);
        assert!(expected2.cooldown_ends_at >= approx_cooldown_ends_at - TimeDelta::seconds(1));

        // check that the returned config instances' states were correctly updated
        assert_eq!(expected1, actual1);
        assert_eq!(expected2, actual2);
    }

    #[tokio::test]
    async fn remove_1() {
        // define the config instance
        let filepath = "/test/filepath".to_string();
        let cfg_inst = ConfigInstance {
            relative_filepath: Some(filepath.clone()),
            target_status: TargetStatus::Removed,
            ..Default::default()
        };

        // create the config instance in the cache
        let dir = Dir::create_temp_dir("deploy").await.unwrap();
        let (cfg_inst_cache, _) = ConfigInstanceCache::spawn(16, dir.file("metadata.json"), 1000)
            .await
            .unwrap();
        let (cfg_inst_content_cache, _) = ConfigInstanceContentCache::spawn(16, dir.clone(), 1000)
            .await
            .unwrap();
        cfg_inst_content_cache
            .write(cfg_inst.id.clone(), json!({"speed": 4}), |_, _| false, true)
            .await
            .unwrap();

        // deploy the config instance
        let settings = Settings::default();
        let cfg_insts_to_apply = HashMap::from([(cfg_inst.id.clone(), cfg_inst.clone())]);
        let result = apply(
            cfg_insts_to_apply,
            &cfg_inst_cache,
            &cfg_inst_content_cache,
            &dir,
            &settings,
        )
        .await
        .unwrap();
        assert_eq!(result.len(), 1);
        let actual = result[&cfg_inst.id].clone();

        // define the expected config instance
        let expected = ConfigInstance {
            activity_status: ActivityStatus::Removed,
            ..cfg_inst
        };

        // check that the returned config instances' states were correctly updated
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn rollback_1_deploy_failed_different_config_schemas() {
        // define the instances with DIFFERENT config schemas but the same filepath
        let filepath = "/test/filepath".to_string();
        let to_deploy = ConfigInstance {
            relative_filepath: Some(filepath.clone()),
            target_status: TargetStatus::Deployed,
            activity_status: ActivityStatus::Queued,
            ..Default::default()
        };
        let to_remove = ConfigInstance {
            relative_filepath: Some(filepath.clone()),
            target_status: TargetStatus::Removed,
            activity_status: ActivityStatus::Deployed,
            ..Default::default()
        };

        // create the cache but omit the config instance content
        let dir = Dir::create_temp_dir("deploy").await.unwrap();
        let (cfg_inst_cache, _) = ConfigInstanceCache::spawn(16, dir.file("metadata.json"), 1000)
            .await
            .unwrap();
        cfg_inst_cache
            .write(to_remove.id.clone(), to_remove.clone(), |_, _| false, true)
            .await
            .unwrap();
        cfg_inst_cache
            .write(to_deploy.id.clone(), to_deploy.clone(), |_, _| false, true)
            .await
            .unwrap();
        let (cfg_inst_content_cache, _) = ConfigInstanceContentCache::spawn(16, dir.clone(), 1000)
            .await
            .unwrap();
        let to_remove_data = json!({"speed": 4});
        cfg_inst_content_cache
            .write(
                to_remove.id.clone(),
                to_remove_data.clone(),
                |_, _| false,
                true,
            )
            .await
            .unwrap();

        // deploy the config instance
        let settings = Settings::default();
        let cfg_insts_to_apply = HashMap::from([
            (to_deploy.id.clone(), to_deploy.clone()),
            (to_remove.id.clone(), to_remove.clone()),
        ]);
        let result = apply(
            cfg_insts_to_apply,
            &cfg_inst_cache,
            &cfg_inst_content_cache,
            &dir,
            &settings,
        )
        .await
        .unwrap();
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
        let cooldown = calc_exp_backoff(
            settings.exp_backoff_base_secs,
            2,
            expected_to_deploy.attempts,
            settings.max_cooldown_secs,
        );
        let approx_cooldown_ends_at = Utc::now() + TimeDelta::seconds(cooldown as i64);
        assert!(expected_to_deploy.cooldown_ends_at <= approx_cooldown_ends_at);
        assert!(
            expected_to_deploy.cooldown_ends_at >= approx_cooldown_ends_at - TimeDelta::seconds(1)
        );

        // check that the returned instances' states were correctly updated
        assert_eq!(expected_to_remove, actual_to_remove);
        assert_eq!(expected_to_deploy, actual_to_deploy);
    }

    #[tokio::test]
    async fn rollback_1_deploy_failed_same_config_schema() {
        // define the instances with DIFFERENT config schemas but the same filepath
        let filepath = "/test/filepath".to_string();
        let to_deploy = ConfigInstance {
            relative_filepath: Some(filepath.clone()),
            target_status: TargetStatus::Deployed,
            activity_status: ActivityStatus::Queued,
            ..Default::default()
        };
        let to_remove = ConfigInstance {
            config_schema_id: to_deploy.config_schema_id.clone(),
            relative_filepath: Some(filepath.clone()),
            target_status: TargetStatus::Removed,
            activity_status: ActivityStatus::Deployed,
            ..Default::default()
        };

        // create the cache but omit the config instance content
        let dir = Dir::create_temp_dir("deploy").await.unwrap();
        let (cfg_inst_cache, _) = ConfigInstanceCache::spawn(16, dir.file("metadata.json"), 1000)
            .await
            .unwrap();
        cfg_inst_cache
            .write(to_remove.id.clone(), to_remove.clone(), |_, _| false, true)
            .await
            .unwrap();
        cfg_inst_cache
            .write(to_deploy.id.clone(), to_deploy.clone(), |_, _| false, true)
            .await
            .unwrap();
        let (cfg_inst_content_cache, _) = ConfigInstanceContentCache::spawn(16, dir.clone(), 1000)
            .await
            .unwrap();
        let to_remove_data = json!({"speed": 4});
        cfg_inst_content_cache
            .write(
                to_remove.id.clone(),
                to_remove_data.clone(),
                |_, _| false,
                true,
            )
            .await
            .unwrap();

        // deploy the config instance
        let settings = Settings::default();
        let cfg_insts_to_apply = HashMap::from([
            (to_deploy.id.clone(), to_deploy.clone()),
            (to_remove.id.clone(), to_remove.clone()),
        ]);
        let result = apply(
            cfg_insts_to_apply,
            &cfg_inst_cache,
            &cfg_inst_content_cache,
            &dir,
            &settings,
        )
        .await
        .unwrap();
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
        let cooldown = calc_exp_backoff(
            settings.exp_backoff_base_secs,
            2,
            expected_to_deploy.attempts,
            settings.max_cooldown_secs,
        );
        let approx_cooldown_ends_at = Utc::now() + TimeDelta::seconds(cooldown as i64);
        assert!(expected_to_deploy.cooldown_ends_at <= approx_cooldown_ends_at);
        assert!(
            expected_to_deploy.cooldown_ends_at >= approx_cooldown_ends_at - TimeDelta::seconds(1)
        );

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
        let to_deploy = ConfigInstance {
            relative_filepath: Some(filepath.clone()),
            target_status: TargetStatus::Deployed,
            activity_status: ActivityStatus::Queued,
            ..Default::default()
        };
        let to_remove = ConfigInstance {
            config_schema_id: to_deploy.config_schema_id.clone(),
            relative_filepath: Some(filepath.clone()),
            target_status: TargetStatus::Removed,
            activity_status: ActivityStatus::Deployed,
            ..Default::default()
        };

        // create the cache but omit the config instance content
        let dir = Dir::create_temp_dir("deploy").await.unwrap();
        let (cfg_inst_cache, _) = ConfigInstanceCache::spawn(16, dir.file("metadata.json"), 1000)
            .await
            .unwrap();
        cfg_inst_cache
            .write(to_remove.id.clone(), to_remove.clone(), |_, _| false, true)
            .await
            .unwrap();
        cfg_inst_cache
            .write(to_deploy.id.clone(), to_deploy.clone(), |_, _| false, true)
            .await
            .unwrap();
        let (cfg_inst_content_cache, _) = ConfigInstanceContentCache::spawn(16, dir.clone(), 1000)
            .await
            .unwrap();
        let to_remove_data = json!({"speed": 4});
        cfg_inst_content_cache
            .write(
                to_remove.id.clone(),
                to_remove_data.clone(),
                |_, _| false,
                true,
            )
            .await
            .unwrap();
        let to_deploy_data = json!({"speed": 5});
        cfg_inst_content_cache
            .write(
                to_deploy.id.clone(),
                to_deploy_data.clone(),
                |_, _| false,
                true,
            )
            .await
            .unwrap();

        // deploy the config instance
        let settings = Settings::default();
        let cfg_insts_to_apply = HashMap::from([
            (to_deploy.id.clone(), to_deploy.clone()),
            (to_remove.id.clone(), to_remove.clone()),
        ]);
        let result = apply(
            cfg_insts_to_apply,
            &cfg_inst_cache,
            &cfg_inst_content_cache,
            &dir,
            &settings,
        )
        .await
        .unwrap();
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
        let (cache, _) = FileCache::spawn(16, dir.file("cache.json"), 1000)
            .await
            .unwrap();

        let cfg_inst = ConfigInstance {
            relative_filepath: Some("/test/filepath".to_string()),
            ..Default::default()
        };

        // create a bunch of instances with that don't match
        for i in 0..10 {
            let cfg_inst = ConfigInstance {
                relative_filepath: Some(format!("/test/filepath{i}")),
                activity_status: ActivityStatus::Deployed,
                target_status: TargetStatus::Removed,
                ..Default::default()
            };
            cache
                .write(cfg_inst.id.clone(), cfg_inst, |_, _| false, true)
                .await
                .unwrap();
        }
        let result = find_instances_to_replace(&cfg_inst, &cache).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn one_file_path_match() {
        let dir = Dir::create_temp_dir("apply").await.unwrap();
        let (cache, _) = FileCache::spawn(16, dir.file("cache.json"), 1000)
            .await
            .unwrap();

        let filepath = "/test/filepath".to_string();
        let cfg_inst = ConfigInstance {
            relative_filepath: Some(filepath.clone()),
            ..Default::default()
        };

        // create a valid replacement config instance
        let to_replace = ConfigInstance {
            relative_filepath: Some(filepath.clone()),
            activity_status: ActivityStatus::Deployed,
            target_status: TargetStatus::Removed,
            ..Default::default()
        };
        cache
            .write(
                to_replace.id.clone(),
                to_replace.clone(),
                |_, _| false,
                true,
            )
            .await
            .unwrap();

        let result = find_instances_to_replace(&cfg_inst, &cache).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], to_replace);
    }

    #[tokio::test]
    async fn one_config_schema_match() {
        let dir = Dir::create_temp_dir("apply").await.unwrap();
        let (cache, _) = FileCache::spawn(16, dir.file("cache.json"), 1000)
            .await
            .unwrap();

        let cfg_inst = ConfigInstance {
            ..Default::default()
        };

        // create a valid replacement config instance
        let to_replace = ConfigInstance {
            config_schema_id: cfg_inst.config_schema_id.clone(),
            activity_status: ActivityStatus::Deployed,
            target_status: TargetStatus::Removed,
            ..Default::default()
        };
        cache
            .write(
                to_replace.id.clone(),
                to_replace.clone(),
                |_, _| false,
                true,
            )
            .await
            .unwrap();

        let result = find_instances_to_replace(&cfg_inst, &cache).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], to_replace);
    }

    #[tokio::test]
    async fn multiple_matches() {
        let dir = Dir::create_temp_dir("apply").await.unwrap();
        let (cache, _) = FileCache::spawn(16, dir.file("cache.json"), 1000)
            .await
            .unwrap();

        let cfg_inst = ConfigInstance {
            relative_filepath: Some("/test/filepath".to_string()),
            ..Default::default()
        };

        // create a valid replacement config instance
        let filepath_to_replace = ConfigInstance {
            relative_filepath: cfg_inst.relative_filepath.clone(),
            activity_status: ActivityStatus::Deployed,
            target_status: TargetStatus::Removed,
            ..Default::default()
        };
        cache
            .write(
                filepath_to_replace.id.clone(),
                filepath_to_replace.clone(),
                |_, _| false,
                true,
            )
            .await
            .unwrap();
        let schema_to_replace = ConfigInstance {
            config_schema_id: cfg_inst.config_schema_id.clone(),
            activity_status: ActivityStatus::Deployed,
            target_status: TargetStatus::Removed,
            ..Default::default()
        };
        cache
            .write(
                schema_to_replace.id.clone(),
                schema_to_replace.clone(),
                |_, _| false,
                true,
            )
            .await
            .unwrap();

        let result = find_instances_to_replace(&cfg_inst, &cache).await.unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.contains(&schema_to_replace));
        assert!(result.contains(&filepath_to_replace));
    }

    #[tokio::test]
    async fn conflicting_target_status() {
        let dir = Dir::create_temp_dir("apply").await.unwrap();
        let (cache, _) = FileCache::spawn(16, dir.file("cache.json"), 1000)
            .await
            .unwrap();

        let cfg_inst = ConfigInstance {
            ..Default::default()
        };

        // create a valid replacement config instance
        let to_replace = ConfigInstance {
            config_schema_id: cfg_inst.config_schema_id.clone(),
            activity_status: ActivityStatus::Deployed,
            target_status: TargetStatus::Deployed,
            ..Default::default()
        };
        cache
            .write(
                to_replace.id.clone(),
                to_replace.clone(),
                |_, _| false,
                true,
            )
            .await
            .unwrap();

        let error = find_instances_to_replace(&cfg_inst, &cache)
            .await
            .unwrap_err();
        assert!(matches!(error, DeployErr::ConflictingDeploymentsErr(_)));
    }
}

pub mod find_replacement_func {
    use super::*;

    #[tokio::test]
    async fn no_matches() {
        let dir = Dir::create_temp_dir("apply").await.unwrap();
        let (cache, _) = FileCache::spawn(16, dir.file("cache.json"), 1000)
            .await
            .unwrap();

        let cfg_inst = ConfigInstance {
            ..Default::default()
        };

        // create a bunch of instances with that don't match
        for _ in 0..10 {
            let cfg_inst = ConfigInstance {
                activity_status: ActivityStatus::Queued,
                target_status: TargetStatus::Deployed,
                ..Default::default()
            };
            cache
                .write(cfg_inst.id.clone(), cfg_inst, |_, _| false, true)
                .await
                .unwrap();
        }

        let result = find_replacement(&cfg_inst, &cache).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn one_match_no_cooldown() {
        let dir = Dir::create_temp_dir("apply").await.unwrap();
        let (cache, _) = FileCache::spawn(16, dir.file("cache.json"), 1000)
            .await
            .unwrap();

        let cfg_inst = ConfigInstance {
            ..Default::default()
        };

        // create a valid replacement config instance
        let replacement = ConfigInstance {
            config_schema_id: cfg_inst.config_schema_id.clone(),
            activity_status: ActivityStatus::Queued,
            target_status: TargetStatus::Deployed,
            ..Default::default()
        };
        cache
            .write(
                replacement.id.clone(),
                replacement.clone(),
                |_, _| false,
                true,
            )
            .await
            .unwrap();

        // find the replacement
        let result = find_replacement(&cfg_inst, &cache).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), replacement);
    }

    #[tokio::test]
    async fn one_match_in_cooldown() {
        let dir = Dir::create_temp_dir("apply").await.unwrap();
        let (cache, _) = FileCache::spawn(16, dir.file("cache.json"), 1000)
            .await
            .unwrap();

        let cfg_inst = ConfigInstance {
            ..Default::default()
        };

        // create a valid replacement config instance
        let replacement = ConfigInstance {
            config_schema_id: cfg_inst.config_schema_id.clone(),
            activity_status: ActivityStatus::Queued,
            target_status: TargetStatus::Deployed,
            cooldown_ends_at: Utc::now() + TimeDelta::seconds(10),
            ..Default::default()
        };
        cache
            .write(
                replacement.id.clone(),
                replacement.clone(),
                |_, _| false,
                true,
            )
            .await
            .unwrap();

        // find the replacement
        let result = find_replacement(&cfg_inst, &cache).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), replacement);
    }

    #[tokio::test]
    async fn multiple_matches_no_cooldown() {
        let dir = Dir::create_temp_dir("apply").await.unwrap();
        let (cache, _) = FileCache::spawn(16, dir.file("cache.json"), 1000)
            .await
            .unwrap();

        let cfg_inst = ConfigInstance {
            ..Default::default()
        };

        // create a valid replacement config instance
        let replacement1 = ConfigInstance {
            config_schema_id: cfg_inst.config_schema_id.clone(),
            activity_status: ActivityStatus::Queued,
            target_status: TargetStatus::Deployed,
            cooldown_ends_at: Utc::now() + TimeDelta::seconds(10),
            ..Default::default()
        };
        cache
            .write(
                replacement1.id.clone(),
                replacement1.clone(),
                |_, _| false,
                true,
            )
            .await
            .unwrap();
        let replacement2 = ConfigInstance {
            config_schema_id: cfg_inst.config_schema_id.clone(),
            activity_status: ActivityStatus::Queued,
            target_status: TargetStatus::Deployed,
            cooldown_ends_at: Utc::now() + TimeDelta::seconds(10),
            ..Default::default()
        };
        cache
            .write(
                replacement2.id.clone(),
                replacement2.clone(),
                |_, _| false,
                true,
            )
            .await
            .unwrap();

        // find the replacement
        find_replacement(&cfg_inst, &cache).await.unwrap_err();
    }
}
