// std crates
use std::sync::Arc;

// internal crates
use config_agent::deploy::fsm;
use config_agent::filesys::dir::Dir;
use config_agent::http::errors::{
    ConfigSchemaNotFound as HTTPConfigSchemaNotFound, HTTPErr, MockErr,
};
use config_agent::models::{
    config_instance::{ActivityStatus, ConfigInstance, TargetStatus},
    config_schema::ConfigSchema,
    device::Device,
};
use config_agent::services::{
    config_instances::{get_deployed, get_deployed::GetDeployedArgs},
    errors::ServiceErr,
};
use config_agent::storage::{
    config_instances::{ConfigInstanceCache, ConfigInstanceContentCache},
    device::DeviceFile,
    config_schemas::ConfigSchemaCache,
};
use config_agent::sync::syncer::{Syncer, SyncerArgs};
use config_agent::trace;
use config_agent::utils::CooldownOptions;

// test crates
use crate::http::mock::{
    MockClient, MockDevicesClient, MockCfgSchsClient,
};
use crate::sync::syncer::{create_token_manager, spawn};

// tokio crates
use serde_json::json;
use tokio::task::JoinHandle;

pub async fn create_syncer(
    dir: &Dir,
    http_client: Arc<MockClient>,
) -> (Syncer, JoinHandle<()>) {
    let auth_client = Arc::new(MockDevicesClient::default());
    let (token_mngr, _) = create_token_manager(dir, auth_client.clone()).await;

    // create the caches
    let (cfg_inst_cache, _) = ConfigInstanceCache::spawn(32, dir.file("instances.json"), 1000)
        .await
        .unwrap();
    let (cfg_inst_content_cache, _) =
        ConfigInstanceContentCache::spawn(32, dir.subdir("instances"), 1000)
            .await
            .unwrap();

    let device = Device::default();
    let (device_file, _) = DeviceFile::spawn_with_default(64, dir.file("device.json"), device.clone()).await.unwrap();

    spawn(
        32,
        SyncerArgs {
            device_file: Arc::new(device_file),
            device_id: "device-id".to_string(),
            http_client: http_client.clone(),
            token_mngr: Arc::new(token_mngr),
            cfg_inst_cache: Arc::new(cfg_inst_cache),
            cfg_inst_content_cache: Arc::new(cfg_inst_content_cache),
            deployment_dir: dir.subdir("syncer"),
            fsm_settings: fsm::Settings::default(),
            cooldown_options: CooldownOptions::default(),
            agent_version: device.agent_version.clone(),
        },
    )
    .unwrap()
}

pub mod errors {
    use super::*;

    #[tokio::test]
    async fn config_schema_not_found_from_storage_or_server() {
        // create the caches
        let dir = Dir::create_temp_dir("get_deployed").await.unwrap();
        let (cfg_inst_cache, _) = ConfigInstanceCache::spawn(32, dir.file("instances.json"), 1000)
            .await
            .unwrap();
        let (cfg_inst_content_cache, _) =
            ConfigInstanceContentCache::spawn(32, dir.subdir("instances"), 1000)
                .await
                .unwrap();
        let (schema_cache, _) = ConfigSchemaCache::spawn(32, dir.file("schemas.json"), 1000)
            .await
            .unwrap();

        // create the mock http client
        let mut cfg_sch_client = MockCfgSchsClient::default();
        cfg_sch_client.set_find_one_config_schema(|| {
            Err(HTTPErr::ConfigSchemaNotFound(Box::new(
                HTTPConfigSchemaNotFound {
                    query_params: "".to_string(),
                    trace: trace!(),
                },
            )))
        });

        // create the syncer
        let http_client = MockClient::default();
        let (syncer, _) = create_syncer(
            &dir, Arc::new(http_client)).await;

        // run the test
        let args = GetDeployedArgs {
            device_id: "device-id".to_string(),
            config_type_slug: "config-type-slug".to_string(),
            config_schema_digest: "config-schema-digest".to_string(),
        };
        let result = get_deployed::get_deployed(
            &args,
            &syncer,
            Arc::new(cfg_inst_cache),
            Arc::new(cfg_inst_content_cache),
            &schema_cache,
            &cfg_sch_client,
            "doesntmatter",
        )
        .await;

        // assert the result
        assert!(matches!(
            result,
            Err(ServiceErr::HTTPErr(ref e)) if matches!(e.source, HTTPErr::ConfigSchemaNotFound(_))
        ));
    }

    #[tokio::test]
    async fn config_schema_not_found_from_storage_and_network_connection_error() {
        // create the caches
        let dir = Dir::create_temp_dir("get_deployed").await.unwrap();
        let (cfg_inst_cache, _) = ConfigInstanceCache::spawn(32, dir.file("instances.json"), 1000)
            .await
            .unwrap();
        let (cfg_inst_content_cache, _) =
            ConfigInstanceContentCache::spawn(32, dir.subdir("instances"), 1000)
                .await
                .unwrap();
        let (schema_cache, _) = ConfigSchemaCache::spawn(32, dir.file("schemas.json"), 1000)
            .await
            .unwrap();

        // create the mock http client
        let mut cfg_sch_client = MockCfgSchsClient::default();
        cfg_sch_client.set_find_one_config_schema(|| {
            Err(HTTPErr::MockErr(Box::new(MockErr {
                is_network_connection_error: true,
            })))
        });

        // create the syncer
        let http_client = MockClient::default();
        let (syncer, _) = create_syncer(&dir, Arc::new(http_client)).await;

        // run the test
        let args = GetDeployedArgs {
            device_id: "device-id".to_string(),
            config_type_slug: "config-type-slug".to_string(),
            config_schema_digest: "config-schema-digest".to_string(),
        };
        let result = get_deployed::get_deployed(
            &args,
            &syncer,
            Arc::new(cfg_inst_cache),
            Arc::new(cfg_inst_content_cache),
            &schema_cache,
            &cfg_sch_client,
            "doesntmatter",
        )
        .await;

        // assert the result
        assert!(matches!(result, Err(ServiceErr::ConfigSchemaNotFound(_))));
    }

    #[tokio::test]
    async fn deployed_config_instance_not_found() {
        // create the caches
        let dir = Dir::create_temp_dir("get_deployed").await.unwrap();
        let (cfg_inst_cache, _) = ConfigInstanceCache::spawn(32, dir.file("instances.json"), 1000)
            .await
            .unwrap();
        let (cfg_inst_content_cache, _) =
            ConfigInstanceContentCache::spawn(32, dir.subdir("instances"), 1000)
                .await
                .unwrap();
        let (schema_cache, _) = ConfigSchemaCache::spawn(32, dir.file("schemas.json"), 1000)
            .await
            .unwrap();

        // create the mock http client
        let mut cfg_sch_client = MockCfgSchsClient::default();
        cfg_sch_client.set_find_one_config_schema(|| {
            Err(HTTPErr::MockErr(Box::new(MockErr {
                is_network_connection_error: true,
            })))
        });

        // create the syncer
        let http_client = MockClient::default();
        let (syncer, _) = create_syncer(&dir, Arc::new(http_client)).await;

        // run the test
        let args = GetDeployedArgs {
            device_id: "device-id".to_string(),
            config_type_slug: "config-type-slug".to_string(),
            config_schema_digest: "config-schema-digest".to_string(),
        };
        let result = get_deployed::get_deployed(
            &args,
            &syncer,
            Arc::new(cfg_inst_cache),
            Arc::new(cfg_inst_content_cache),
            &schema_cache,
            &cfg_sch_client,
            "doesntmatter",
        )
        .await;

        // assert the result
        assert!(matches!(result, Err(ServiceErr::ConfigSchemaNotFound(_))));
    }
}

pub mod success {
    use super::*;

    #[tokio::test]
    async fn found_in_storage_network_connection_error() {
        let cfg_sch_id = "cfg-sch-id".to_string();
        let cfg_sch_digest = "cfg-schema-digest".to_string();
        let cfg_type_slug = "cfg-type-slug".to_string();
        let cfg_sch = ConfigSchema {
            id: cfg_sch_id.clone(),
            digest: cfg_sch_digest.clone(),
            config_type_slug: Some(cfg_type_slug.clone()),
            ..Default::default()
        };
        let cfg_inst_id = "cfg-inst-id".to_string();
        let cfg_inst = ConfigInstance {
            id: cfg_inst_id.clone(),
            target_status: TargetStatus::Deployed,
            activity_status: ActivityStatus::Queued,
            config_schema_id: cfg_sch_id.clone(),
            ..Default::default()
        };

        // create the caches
        let dir = Dir::create_temp_dir("get_deployed").await.unwrap();
        let (cfg_inst_cache, _) = ConfigInstanceCache::spawn(32, dir.file("instances.json"), 1000)
            .await
            .unwrap();
        cfg_inst_cache
            .write(cfg_inst_id.clone(), cfg_inst.clone(), |_, _| false, true)
            .await
            .unwrap();
        let (cfg_inst_content_cache, _) =
            ConfigInstanceContentCache::spawn(32, dir.subdir("instances"), 1000)
                .await
                .unwrap();
        cfg_inst_content_cache
            .write(cfg_inst_id.clone(), json!({}), |_, _| false, true)
            .await
            .unwrap();
        let (schema_cache, _) = ConfigSchemaCache::spawn(32, dir.file("schemas.json"), 1000)
            .await
            .unwrap();
        schema_cache
            .write(cfg_sch_id.clone(), cfg_sch.clone(), |_, _| false, true)
            .await
            .unwrap();

        // create the mock http client
        let cfg_sch_client = MockCfgSchsClient::default();

        // create the syncer
        let http_client = Arc::new(MockClient::default());
        http_client.set_list_all_config_instances(|| {
            Err(HTTPErr::MockErr(Box::new(MockErr {
                is_network_connection_error: true,
            })))
        });
        let (syncer, _) = create_syncer(&dir, http_client.clone()).await;

        // run the test
        let args = GetDeployedArgs {
            device_id: "device-id".to_string(),
            config_type_slug: cfg_type_slug.clone(),
            config_schema_digest: cfg_sch_digest.clone(),
        };
        let deployed_inst = get_deployed::get_deployed(
            &args,
            &syncer,
            Arc::new(cfg_inst_cache),
            Arc::new(cfg_inst_content_cache),
            &schema_cache,
            &cfg_sch_client,
            "doesntmatter",
        )
        .await
        .unwrap();

        assert_eq!(deployed_inst.id, cfg_inst_id);
    }

    #[tokio::test]
    async fn pull_and_deploy_unknown_from_server() {
        // create the caches
        let dir = Dir::create_temp_dir("get_deployed").await.unwrap();
        let (cfg_inst_cache, _) = ConfigInstanceCache::spawn(32, dir.file("instances.json"), 1000)
            .await
            .unwrap();
        let (cfg_inst_content_cache, _) =
            ConfigInstanceContentCache::spawn(32, dir.subdir("instances"), 1000)
                .await
                .unwrap();
        let (schema_cache, _) = ConfigSchemaCache::spawn(32, dir.file("schemas.json"), 1000)
            .await
            .unwrap();

        let cfg_sch_id = "cfg-sch-id".to_string();
        let cfg_sch_digest = "cfg-schema-digest".to_string();
        let cfg_type_slug = "cfg-type-slug".to_string();
        let cfg_sch = openapi_client::models::ConfigSchema {
            id: cfg_sch_id.clone(),
            digest: cfg_sch_digest.clone(),
            ..Default::default()
        };
        let cfg_inst_id = "cfg-inst-id".to_string();
        let cfg_inst = openapi_client::models::ConfigInstance {
            id: cfg_inst_id.clone(),
            target_status: openapi_client::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_DEPLOYED,
            activity_status: openapi_client::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_QUEUED,
            config_schema_id: cfg_sch_id.clone(),
            content: Some(json!({})),
            ..Default::default()
        };

        // create the mock http client
        let mut cfg_sch_client = MockCfgSchsClient::default();
        let cfg_sch_cloned = cfg_sch.clone();
        cfg_sch_client.set_find_one_config_schema(move || Ok(cfg_sch_cloned.clone()));

        // create the syncer
        let http_client = MockClient::default();
        let cfg_inst_cloned = cfg_inst.clone();
        http_client.set_list_all_config_instances(move || Ok(vec![cfg_inst_cloned.clone()]));
        let (syncer, _) = create_syncer(&dir, Arc::new(http_client)).await;

        // run the test
        let args = GetDeployedArgs {
            device_id: "device-id".to_string(),
            config_type_slug: cfg_type_slug.clone(),
            config_schema_digest: cfg_sch_digest.clone(),
        };
        let deployed_inst = get_deployed::get_deployed(
            &args,
            &syncer,
            Arc::new(cfg_inst_cache),
            Arc::new(cfg_inst_content_cache),
            &schema_cache,
            &cfg_sch_client,
            "doesntmatter",
        )
        .await
        .unwrap();

        assert_eq!(deployed_inst.id, cfg_inst_id);
    }
}
