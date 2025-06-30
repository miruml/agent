// internal crates
use config_agent::cache::file::FileCache;
use config_agent::deploy::{
    filesys::deploy_with_rollback,
    observer::Observer,
    fsm::Settings,
    fsm,
};
use config_agent::filesys::path::PathExt;
use config_agent::models::config_instance::{
    ConfigInstance,
    ActivityStatus,
    ErrorStatus,
    TargetStatus,
};
use config_agent::filesys::dir::Dir;
use crate::deploy::observer::HistoryObserver;

// external crates
use chrono::{Utc, TimeDelta};
use serde_json::json;


pub mod deploy_with_rollback {
    use super::*;

    #[tokio::test]
    async fn deploy_1_failed_missing_instance_data() {
        // define the instance 
        let filepath = "/test/filepath".to_string();
        let instance = ConfigInstance {
            filepath: Some(filepath.clone()),
            // target status must be deployed to increment failure attempts
            target_status: TargetStatus::Deployed,
            ..Default::default()
        };

        // create the cache but omit the instance data
        let dir = Dir::create_temp_dir("deploy").await.unwrap();
        let (cache, _) = FileCache::spawn(
            dir.file("cache.json"), 16,
        ).await.unwrap();

        // deploy the instance
        let settings = Settings::default();
        let mut observers: Vec<&mut dyn Observer> = Vec::new();
        let mut observer = HistoryObserver::new();
        observers.push(&mut observer);
        let (deploy_results, result) = deploy_with_rollback(
            vec![],
            vec![instance.clone()],
            &cache,
            &dir,
            &settings,
            &mut observers,
        ).await;
        result.unwrap();

        // define the expected instance
        let expected = ConfigInstance {
            activity_status: ActivityStatus::Removed,
            error_status: ErrorStatus::Retrying,
            attempts: 1,
            cooldown_ends_at: deploy_results.to_deploy[0].cooldown_ends_at,
            ..instance
        };
        let cooldown = fsm::calc_exp_backoff(
            2,
            settings.exp_backoff_base_secs,
            expected.attempts,
            settings.max_cooldown_secs,
        );
        let approx_cooldown_ends_at = Utc::now() + TimeDelta::seconds(cooldown as i64);
        assert!(expected.cooldown_ends_at <= approx_cooldown_ends_at);
        assert!(expected.cooldown_ends_at >= approx_cooldown_ends_at - TimeDelta::seconds(1));

        // check that the returned instances' states were correctly updated
        assert!(deploy_results.to_remove.is_empty());
        assert_eq!(deploy_results.to_deploy.len(), 1);
        assert_eq!(deploy_results.to_deploy[0], expected);

        // check that the observer's history was correctly updated
        assert_eq!(observer.history.len(), 2);
        assert_eq!(observer.history[1], expected);
    }

    #[tokio::test]
    async fn deploy_1_no_filepath() {
        // define the instance 
        let instance = ConfigInstance {
            filepath: None,
            ..Default::default()
        };

        // create the instance in the cache
        let dir = Dir::create_temp_dir("deploy").await.unwrap();
        let (cache, _) = FileCache::spawn(
            dir.file("cache.json"), 16,
        ).await.unwrap();
        cache.write(
            instance.id.clone(), json!({"speed": 4}), |_, _| false, true,
        ).await.unwrap();

        // deploy the instance
        let settings = Settings::default();
        let mut observers: Vec<&mut dyn Observer> = Vec::new();
        let mut observer = HistoryObserver::new();
        observers.push(&mut observer);
        let (deploy_results, result) = deploy_with_rollback(
            vec![],
            vec![instance.clone()],
            &cache,
            &dir,
            &settings,
            &mut observers,
        ).await;
        result.unwrap();

        // define the expected instance
        let expected = ConfigInstance {
            activity_status: ActivityStatus::Deployed,
            ..instance
        };

        // check that the returned instances' states were correctly updated
        assert!(deploy_results.to_remove.is_empty());
        assert_eq!(deploy_results.to_deploy.len(), 1);
        assert_eq!(deploy_results.to_deploy[0], expected);

        // check that the observer's history was correctly updated
        assert_eq!(observer.history.len(), 1);
        assert_eq!(observer.history[0], expected);
    }

    // deploy 1 - filepath specified overwrites existing file
    #[tokio::test]
    async fn deploy_1_filepath_specified_overwrite_existing() {
        // define the instance
        let filepath = "/test/filepath".to_string();
        let instance = ConfigInstance {
            filepath: Some(filepath.clone()),
            ..Default::default()
        };

        // create the instance in the cache
        let dir = Dir::create_temp_dir("deploy").await.unwrap();
        let (cache, _) = FileCache::spawn(
            dir.file("cache.json"), 100,
        ).await.unwrap();
        let instance_data = json!({"speed": 4});
        cache.write(
            instance.id.clone(), instance_data.clone(), |_, _| false, true,
        ).await.unwrap();

        // create the file in the deployment directory
        let file = dir.file(filepath.as_str());
        file.write_json(&instance_data, true, true).await.unwrap();

        // deploy the instance
        let settings = Settings::default();
        let mut observers: Vec<&mut dyn Observer> = Vec::new();
        let mut observer = HistoryObserver::new();
        observers.push(&mut observer);
        let (deploy_results, result) = deploy_with_rollback(
            vec![],
            vec![instance.clone()],
            &cache,
            &dir,
            &settings,
            &mut observers
        ).await;
        result.unwrap();

        // define the expected instance
        let expected = ConfigInstance {
            activity_status: ActivityStatus::Deployed,
            ..instance
        };

        // check that the returned instances' states were correctly updated
        assert!(deploy_results.to_remove.is_empty());
        assert_eq!(deploy_results.to_deploy.len(), 1);
        assert_eq!(deploy_results.to_deploy[0], expected);

        // check that the observer's history was correctly updated
        assert_eq!(observer.history.len(), 1);
        assert_eq!(observer.history[0], expected);

        // check that the file was created
        let file = dir.file(filepath.as_str());
        let actual = file.read_json::<serde_json::Value>().await.unwrap();
        assert_eq!(actual, instance_data);
    }

    // deploy 1 - filepath specified doesn't overwrite existing file
    #[tokio::test]
    async fn deploy_1_filepath_specified_no_overwrite() {
        // define the instance
        let filepath = "/test/filepath".to_string();
        let instance = ConfigInstance {
            filepath: Some(filepath.clone()),
            ..Default::default()
        };

        // create the instance in the cache
        let dir = Dir::create_temp_dir("deploy").await.unwrap();
        let (cache, _) = FileCache::spawn(
            dir.file("cache.json"), 100,
        ).await.unwrap();
        let instance_data = json!({"speed": 4});
        cache.write(
            instance.id.clone(), instance_data.clone(), |_, _| false, true,
        ).await.unwrap();

        // deploy the instance
        let settings = Settings::default();
        let mut observers: Vec<&mut dyn Observer> = Vec::new();
        let mut observer = HistoryObserver::new();
        observers.push(&mut observer);
        let (deploy_results, result) = deploy_with_rollback(
            vec![],
            vec![instance.clone()],
            &cache,
            &dir,
            &settings,
            &mut observers
        ).await;
        result.unwrap();

        // define the expected instance
        let expected = ConfigInstance {
            activity_status: ActivityStatus::Deployed,
            ..instance
        };

        // check that the returned instances' states were correctly updated
        assert!(deploy_results.to_remove.is_empty());
        assert_eq!(deploy_results.to_deploy.len(), 1);
        assert_eq!(deploy_results.to_deploy[0], expected);

        // check that the observer's history was correctly updated
        assert_eq!(observer.history.len(), 1);
        assert_eq!(observer.history[0], expected);

        // check that the file was created
        let file = dir.file(filepath.as_str());
        let actual = file.read_json::<serde_json::Value>().await.unwrap();
        assert_eq!(actual, instance_data);
    }

    // remove failures are essentially impossible since removing a file that doesn't exist
    // does not throw an error

    #[tokio::test]
    async fn remove_1_no_filepath() {
        // define the instance 
        let instance = ConfigInstance {
            filepath: None,
            ..Default::default()
        };

        // create the instance in the cache
        let dir = Dir::create_temp_dir("deploy").await.unwrap();
        let (cache, _) = FileCache::spawn(
            dir.file("cache.json"), 16,
        ).await.unwrap();
        cache.write(
            instance.id.clone(), json!({"speed": 4}), |_, _| false, true,
        ).await.unwrap();

        // deploy the instance
        let settings = Settings::default();
        let mut observers: Vec<&mut dyn Observer> = Vec::new();
        let mut observer = HistoryObserver::new();
        observers.push(&mut observer);
        let (deploy_results, result) = deploy_with_rollback(
            vec![instance.clone()],
            vec![],
            &cache,
            &dir,
            &settings,
            &mut observers,
        ).await;
        result.unwrap();

        // define the expected instance
        let expected = ConfigInstance {
            activity_status: ActivityStatus::Removed,
            ..instance
        };

        // check that the returned instances' states were correctly updated
        assert_eq!(deploy_results.to_remove.len(), 1);
        assert_eq!(deploy_results.to_remove[0], expected);
        assert!(deploy_results.to_deploy.is_empty());

        // check that the observer's history was correctly updated
        assert_eq!(observer.history.len(), 1);
        assert_eq!(observer.history[0], expected);
    }

    #[tokio::test]
    async fn remove_1_filepath_specified_doesnt_exist() {
        // define the instance 
        let filepath = "/test/filepath".to_string();
        let instance = ConfigInstance {
            filepath: Some(filepath.clone()),
            ..Default::default()
        };

        // create the instance in the cache
        let dir = Dir::create_temp_dir("deploy").await.unwrap();
        let (cache, _) = FileCache::spawn(
            dir.file("cache.json"), 16,
        ).await.unwrap();
        cache.write(
            instance.id.clone(), json!({"speed": 4}), |_, _| false, true,
        ).await.unwrap();

        // deploy the instance
        let settings = Settings::default();
        let mut observers: Vec<&mut dyn Observer> = Vec::new();
        let mut observer = HistoryObserver::new();
        observers.push(&mut observer);
        let (deploy_results, result) = deploy_with_rollback(
            vec![instance.clone()],
            vec![],
            &cache,
            &dir,
            &settings,
            &mut observers,
        ).await;
        result.unwrap();

        // define the expected instance
        let expected = ConfigInstance {
            activity_status: ActivityStatus::Removed,
            ..instance
        };

        // check that the returned instances' states were correctly updated
        assert_eq!(deploy_results.to_remove.len(), 1);
        assert_eq!(deploy_results.to_remove[0], expected);
        assert!(deploy_results.to_deploy.is_empty());

        // check that the observer's history was correctly updated
        assert_eq!(observer.history.len(), 1);
        assert_eq!(observer.history[0], expected);
    }

    #[tokio::test]
    async fn remove_1_filepath_specified_exists() {
        // define the instance 
        let filepath = "/test/filepath".to_string();
        let instance = ConfigInstance {
            filepath: Some(filepath.clone()),
            ..Default::default()
        };

        // create the instance in the cache
        let dir = Dir::create_temp_dir("deploy").await.unwrap();
        let (cache, _) = FileCache::spawn(
            dir.file("cache.json"), 16,
        ).await.unwrap();
        let instance_data = json!({"speed": 4});
        cache.write(
            instance.id.clone(), instance_data.clone(), |_, _| false, true,
        ).await.unwrap();

        // create the file in the deployment directory
        let file = dir.file(filepath.as_str());
        file.write_json(&instance_data, true, true).await.unwrap();

        // deploy the instance
        let settings = Settings::default();
        let mut observers: Vec<&mut dyn Observer> = Vec::new();
        let mut observer = HistoryObserver::new();
        observers.push(&mut observer);
        let (deploy_results, result) = deploy_with_rollback(
            vec![instance.clone()],
            vec![],
            &cache,
            &dir,
            &settings,
            &mut observers,
        ).await;
        result.unwrap();

        // define the expected instance
        let expected = ConfigInstance {
            activity_status: ActivityStatus::Removed,
            ..instance
        };

        // check that the returned instances' states were correctly updated
        assert_eq!(deploy_results.to_remove.len(), 1);
        assert_eq!(deploy_results.to_remove[0], expected);
        assert!(deploy_results.to_deploy.is_empty());

        // check that the observer's history was correctly updated
        assert_eq!(observer.history.len(), 1);
        assert_eq!(observer.history[0], expected);

        // check that the file was removed
        assert!(!file.exists());
    }

    #[tokio::test]
    async fn rollback_1_deploy_missing_instance_data() {
        // define the instance 
        let to_deploy_filepath = "/to/deploy/filepath".to_string();
        let to_deploy = ConfigInstance {
            filepath: Some(to_deploy_filepath.clone()),
            // target status must be deployed to increment failure attempts
            target_status: TargetStatus::Deployed,
            ..Default::default()
        };
        let to_remove_filepath = "/to/remove/filepath".to_string();
        let to_remove = ConfigInstance {
            filepath: Some(to_remove_filepath.clone()),
            target_status: TargetStatus::Removed,
            ..Default::default()
        };

        // create the cache but the instance data for the to_deploy instance
        let dir = Dir::create_temp_dir("deploy").await.unwrap();
        let (cache, _) = FileCache::spawn(
            dir.file("cache.json"), 16,
        ).await.unwrap();
        let to_remove_data = json!({"speed": 8});
        cache.write(
            to_remove.id.clone(), to_remove_data.clone(), |_, _| false, true,
        ).await.unwrap();

        // deploy the instance
        let settings = Settings::default();
        let mut observers: Vec<&mut dyn Observer> = Vec::new();
        let mut observer = HistoryObserver::new();
        observers.push(&mut observer);
        let (deploy_results, result) = deploy_with_rollback(
            vec![to_remove.clone()],
            vec![to_deploy.clone()],
            &cache,
            &dir,
            &settings,
            &mut observers,
        ).await;
        result.unwrap();

        // define the expected instances
        let expected_to_remove = ConfigInstance {
            activity_status: ActivityStatus::Deployed,
            ..to_remove
        };

        let expected_to_deploy = ConfigInstance {
            activity_status: ActivityStatus::Removed,
            error_status: ErrorStatus::Retrying,
            attempts: 1,
            cooldown_ends_at: deploy_results.to_deploy[0].cooldown_ends_at,
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
        assert_eq!(deploy_results.to_remove.len(), 1);
        assert_eq!(deploy_results.to_remove[0], expected_to_remove);
        assert_eq!(deploy_results.to_deploy.len(), 1);
        assert_eq!(deploy_results.to_deploy[0], expected_to_deploy);

        // check that the observer's history was correctly updated
        assert_eq!(observer.history.len(), 4);
        assert_eq!(observer.history[2], expected_to_deploy);
        assert_eq!(observer.history[3], expected_to_remove);

        // check that the removed instance is still deployed
        let file = dir.file(to_remove_filepath.as_str());
        let actual = file.read_json::<serde_json::Value>().await.unwrap();
        assert_eq!(actual, to_remove_data);
    }

    #[tokio::test]
    async fn rollback_n_deploy_missing_instance_data() {
        // define the instances 
        let n = 10;
        let mut to_deploy_instances = Vec::new();
        for i in 0..n {
            let filepath = format!("/to/deploy/filepath-{}", i);
            let instance = ConfigInstance {
                filepath: Some(filepath.clone()),
                target_status: TargetStatus::Deployed,
                ..Default::default()
            };
            to_deploy_instances.push(instance);
        }
        let mut to_remove_instances = Vec::new();
        for i in 0..n {
            let filepath = format!("/to/remove/filepath-{}", i);
            let instance = ConfigInstance {
                filepath: Some(filepath.clone()),
                target_status: TargetStatus::Removed,
                ..Default::default()
            };
            to_remove_instances.push(instance);
        }

        // create the cache but the instance data for the to_deploy instance
        let dir = Dir::create_temp_dir("deploy").await.unwrap();
        let (cache, _) = FileCache::spawn(
            dir.file("cache.json"), 16,
        ).await.unwrap();
        for instance in to_remove_instances.iter() {
            cache.write(
                instance.id.clone(), json!({"filepath": instance.filepath.clone()}), |_, _| false, true,
            ).await.unwrap();
        }

        // deploy the instance
        let settings = Settings::default();
        let mut observers: Vec<&mut dyn Observer> = Vec::new();
        let mut observer = HistoryObserver::new();
        observers.push(&mut observer);
        let (deploy_results, result) = deploy_with_rollback(
            to_remove_instances.clone(),
            to_deploy_instances.clone(),
            &cache,
            &dir,
            &settings,
            &mut observers,
        ).await;
        result.unwrap();

        // define the expected instances
        let mut expected_to_remove_instances = to_remove_instances.clone();
        for instance in expected_to_remove_instances.iter_mut() {
            instance.activity_status = ActivityStatus::Deployed;
        }
        let mut expected_to_deploy_instances = to_deploy_instances.clone();
        for (i, instance) in expected_to_deploy_instances.iter_mut().enumerate() {
            instance.activity_status = ActivityStatus::Removed;
            if i == 0 {
                instance.error_status = ErrorStatus::Retrying;
                instance.attempts = 1;
                let cooldown = fsm::calc_exp_backoff(
                    2,
                    settings.exp_backoff_base_secs,
                    instance.attempts,
                    settings.max_cooldown_secs,
                );
                let approx_cooldown_ends_at = Utc::now() + TimeDelta::seconds(cooldown as i64);
                instance.cooldown_ends_at = deploy_results.to_deploy[i].cooldown_ends_at;
                assert!(instance.cooldown_ends_at <= approx_cooldown_ends_at);
                assert!(instance.cooldown_ends_at >= approx_cooldown_ends_at - TimeDelta::seconds(1));
            }
        }

        // check that the returned instances' states were correctly updated
        assert_eq!(deploy_results.to_remove.len(), n);
        assert_eq!(deploy_results.to_remove.len(), expected_to_remove_instances.len());
        for (i, instance) in deploy_results.to_remove.iter().enumerate() {
            assert_eq!(instance, &expected_to_remove_instances[i]);
        }
        assert_eq!(deploy_results.to_deploy.len(), n);
        assert_eq!(deploy_results.to_deploy.len(), expected_to_deploy_instances.len());
        for (i, instance) in deploy_results.to_deploy.iter().enumerate() {
            assert_eq!(instance, &expected_to_deploy_instances[i]);
        }

        // check that the observer's history was correctly updated
        for (i, instance) in expected_to_deploy_instances.iter().enumerate() {
            assert_eq!(&observer.history[n+1+i], instance);
        }
        for (i, instance) in expected_to_remove_instances.iter().enumerate() {
            assert_eq!(&observer.history[2*n+1+i], instance);
        }

        // check that the removed instances are still deployed
        for instance in to_remove_instances {
            let file = dir.file(instance.filepath.as_ref().unwrap());
            let actual = file.read_json::<serde_json::Value>().await.unwrap();
            assert_eq!(actual, json!({"filepath": instance.filepath.as_ref().unwrap()}));
        }
    }
}