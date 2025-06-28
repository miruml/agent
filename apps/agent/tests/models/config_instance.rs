// standard library
use std::collections::HashSet;

// internal crates
use config_agent::models::config_instance::{
    ConfigInstance,
    ActivityStatus,
    ErrorStatus,
    Status,
    TargetStatus,
};
use openapi_client::models::{
    BackendConfigInstance,
    ConfigInstanceActivityStatus,
    ConfigInstanceErrorStatus,
    ConfigInstanceStatus,
    ConfigInstanceTargetStatus,
};

// external crates
use chrono::{DateTime, TimeDelta, Utc};
use serde_json::json;
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

#[test]
fn deserialize_target_status() {
    struct TestCase {
        input: &'static str,
        expected: TargetStatus,
    }

    let test_cases = vec![
        // valid
        TestCase {
            input: "\"created\"",
            expected: TargetStatus::Created,
        },
        TestCase {
            input: "\"deployed\"",
            expected: TargetStatus::Deployed,
        },
        TestCase {
            input: "\"removed\"",
            expected: TargetStatus::Removed,
        },
        // default
        TestCase {
            input: "\"unknown\"",
            expected: TargetStatus::Created,
        },
    ];

    let mut variants = TargetStatus::variants().into_iter().collect::<HashSet<_>>();

    for test_case in test_cases {
        variants.remove(&test_case.expected);
        let deserialized = serde_json::from_str::<TargetStatus>(test_case.input).unwrap();
        assert_eq!(deserialized, test_case.expected);
    }

    assert!(variants.is_empty(), "variants: {:?}", variants);
}

#[test]
fn target_status_backend_and_sdk_conversions() {
    struct TestCase {
        storage: TargetStatus,
        backend: openapi_client::models::ConfigInstanceTargetStatus,
        sdk: openapi_server::models::ConfigInstanceTargetStatus,
    }

    let test_cases = vec![
        TestCase {
            storage: TargetStatus::Created,
            backend: openapi_client::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_CREATED,
            sdk: openapi_server::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_CREATED,
        },
        TestCase {
            storage: TargetStatus::Deployed,
            backend: openapi_client::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_DEPLOYED,
            sdk: openapi_server::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_DEPLOYED,
        },
        TestCase {
            storage: TargetStatus::Removed,
            backend: openapi_client::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_REMOVED,
            sdk: openapi_server::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_REMOVED,
        },
    ];

    let mut variants = TargetStatus::variants().into_iter().collect::<HashSet<_>>();

    for test_case in test_cases {
        variants.remove(&test_case.storage);
        assert_eq!(TargetStatus::from_backend(&test_case.backend), test_case.storage);
        assert_eq!(test_case.backend, TargetStatus::to_backend(&test_case.storage));
        assert_eq!(TargetStatus::from_sdk(&test_case.sdk), test_case.storage);
        assert_eq!(test_case.sdk, TargetStatus::to_sdk(&test_case.storage));
    }

    assert!(variants.is_empty(), "variants: {:?}", variants);
}

#[test]
fn deserialize_activity_status() {
    struct TestCase {
        input: &'static str,
        expected: ActivityStatus,
    }

    let test_cases = vec![
        // valid
        TestCase {
            input: "\"created\"",
            expected: ActivityStatus::Created,
        },
        TestCase {
            input: "\"queued\"",
            expected: ActivityStatus::Queued,
        },
        TestCase {
            input: "\"deployed\"",
            expected: ActivityStatus::Deployed,
        },
        TestCase {
            input: "\"removed\"",
            expected: ActivityStatus::Removed,
        },
        TestCase {
            input: "\"unknown\"",
            expected: ActivityStatus::Created,
        },
    ];

    let mut variants = ActivityStatus::variants().into_iter().collect::<HashSet<_>>();

    for test_case in test_cases {
        variants.remove(&test_case.expected);
        let deserialized = serde_json::from_str::<ActivityStatus>(test_case.input).unwrap();
        assert_eq!(deserialized, test_case.expected);
    }

    assert!(variants.is_empty(), "variants: {:?}", variants);
}

#[test]
fn activity_status_backend_and_sdk_conversions() {
    struct TestCase {
        storage: ActivityStatus,
        backend: openapi_client::models::ConfigInstanceActivityStatus,
        sdk: openapi_server::models::ConfigInstanceActivityStatus,
    }

    let test_cases = vec![
        TestCase {
            storage: ActivityStatus::Created,
            backend: openapi_client::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_CREATED,
            sdk: openapi_server::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_CREATED,
        },
        TestCase {
            storage: ActivityStatus::Queued,
            backend: openapi_client::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_QUEUED,
            sdk: openapi_server::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_QUEUED,
        },
        TestCase {
            storage: ActivityStatus::Deployed,
            backend: openapi_client::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_DEPLOYED,
            sdk: openapi_server::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_DEPLOYED,
        },
        TestCase {
            storage: ActivityStatus::Removed,
            backend: openapi_client::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_REMOVED,
            sdk: openapi_server::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_REMOVED,
        },
    ];

    let mut variants = ActivityStatus::variants().into_iter().collect::<HashSet<_>>();
    
    for test_case in test_cases {
        variants.remove(&test_case.storage);
        assert_eq!(ActivityStatus::from_backend(&test_case.backend), test_case.storage);
        assert_eq!(test_case.backend, ActivityStatus::to_backend(&test_case.storage));
        assert_eq!(ActivityStatus::from_sdk(&test_case.sdk), test_case.storage);
        assert_eq!(test_case.sdk, ActivityStatus::to_sdk(&test_case.storage));
    }

    assert!(variants.is_empty(), "variants: {:?}", variants);
}

#[test]
fn deserialize_error_status() {
    struct TestCase {
        input: &'static str,
        expected: ErrorStatus,
    }

    let test_cases = vec![
        // valid
        TestCase {
            input: "\"none\"",
            expected: ErrorStatus::None,
        },
        TestCase {
            input: "\"failed\"",
            expected: ErrorStatus::Failed,
        },
        TestCase {
            input: "\"retrying\"",
            expected: ErrorStatus::Retrying,
        },
        // default
        TestCase {
            input: "\"unknown\"",
            expected: ErrorStatus::None,
        },
    ];

    let mut variants = ErrorStatus::variants().into_iter().collect::<HashSet<_>>();

    for test_case in test_cases {
        variants.remove(&test_case.expected);
        let deserialized = serde_json::from_str::<ErrorStatus>(test_case.input).unwrap();
        assert_eq!(deserialized, test_case.expected);
    }
}

#[test]
fn error_status_backend_and_sdk_conversions() {
    struct TestCase {
        storage: ErrorStatus,
        backend: openapi_client::models::ConfigInstanceErrorStatus,
        sdk: openapi_server::models::ConfigInstanceErrorStatus,
    }

    let test_cases = vec![
        TestCase {
            storage: ErrorStatus::None,
            backend: openapi_client::models::ConfigInstanceErrorStatus::CONFIG_INSTANCE_ERROR_STATUS_NONE,
            sdk: openapi_server::models::ConfigInstanceErrorStatus::CONFIG_INSTANCE_ERROR_STATUS_NONE,
        },
        TestCase {
            storage: ErrorStatus::Failed,
            backend: openapi_client::models::ConfigInstanceErrorStatus::CONFIG_INSTANCE_ERROR_STATUS_FAILED,
            sdk: openapi_server::models::ConfigInstanceErrorStatus::CONFIG_INSTANCE_ERROR_STATUS_FAILED,
        },
        TestCase {
            storage: ErrorStatus::Retrying,
            backend: openapi_client::models::ConfigInstanceErrorStatus::CONFIG_INSTANCE_ERROR_STATUS_RETRYING,
            sdk: openapi_server::models::ConfigInstanceErrorStatus::CONFIG_INSTANCE_ERROR_STATUS_RETRYING,
        },
    ];

    let mut variants = ErrorStatus::variants().into_iter().collect::<HashSet<_>>();

    for test_case in test_cases {
        variants.remove(&test_case.storage);
        assert_eq!(ErrorStatus::from_backend(&test_case.backend), test_case.storage);
        assert_eq!(test_case.backend, ErrorStatus::to_backend(&test_case.storage));
        assert_eq!(ErrorStatus::from_sdk(&test_case.sdk), test_case.storage);
        assert_eq!(test_case.sdk, ErrorStatus::to_sdk(&test_case.storage));
    }

    assert!(variants.is_empty(), "variants: {:?}", variants);
}

#[test]
fn deserialize_status() {
    struct TestCase {
        input: &'static str,
        expected: Status,
    }

    let test_cases = vec![
        // valid
        TestCase {
            input: "\"created\"",
            expected: Status::Created,
        },
        TestCase {
            input: "\"queued\"",
            expected: Status::Queued,
        },
        TestCase {
            input: "\"deployed\"",
            expected: Status::Deployed,
        },
        TestCase {
            input: "\"removed\"",
            expected: Status::Removed,
        },
        TestCase {
            input: "\"failed\"",
            expected: Status::Failed,
        },
        TestCase {
            input: "\"retrying\"",
            expected: Status::Retrying,
        },
        // default
        TestCase {
            input: "\"unknown\"",
            expected: Status::Created,
        },
    ];

    let mut variants = Status::variants().into_iter().collect::<HashSet<_>>();

    for test_case in test_cases {
        variants.remove(&test_case.expected);
        let deserialized = serde_json::from_str::<Status>(test_case.input).unwrap();
        assert_eq!(deserialized, test_case.expected);
    }

    assert!(variants.is_empty(), "variants: {:?}", variants);
}

#[test]
fn status_backend_and_sdk_conversions() {
    struct TestCase {
        storage: Status,
        backend: openapi_client::models::ConfigInstanceStatus,
        sdk: openapi_server::models::ConfigInstanceStatus,
    }

    let test_cases = vec![
        TestCase {
            storage: Status::Created,
            backend: openapi_client::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_CREATED,
            sdk: openapi_server::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_CREATED,
        },
        TestCase {
            storage: Status::Queued,
            backend: openapi_client::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_QUEUED,
            sdk: openapi_server::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_QUEUED,
        },
        TestCase {
            storage: Status::Deployed,
            backend: openapi_client::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_DEPLOYED,
            sdk: openapi_server::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_DEPLOYED,
        },
        TestCase {
            storage: Status::Removed,
            backend: openapi_client::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_REMOVED,
            sdk: openapi_server::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_REMOVED,
        },
        TestCase {
            storage: Status::Failed,
            backend: openapi_client::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_FAILED,
            sdk: openapi_server::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_FAILED,
        },
        TestCase {
            storage: Status::Retrying,
            backend: openapi_client::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_RETRYING,
            sdk: openapi_server::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_RETRYING,
        },
    ];

    let mut variants = Status::variants().into_iter().collect::<HashSet<_>>();

    for test_case in test_cases {
        variants.remove(&test_case.storage);
        assert_eq!(Status::from_backend(&test_case.backend), test_case.storage);
        assert_eq!(test_case.backend, Status::to_backend(&test_case.storage));
        assert_eq!(Status::from_sdk(&test_case.sdk), test_case.storage);
        assert_eq!(test_case.sdk, Status::to_sdk(&test_case.storage));
    }

    assert!(variants.is_empty(), "variants: {:?}", variants);
}

#[test]
fn deserialize_config_instance() {
    // valid deserialization
    let expected = ConfigInstance {
        id: "123".to_string(),
        target_status: TargetStatus::Created,
        activity_status: ActivityStatus::Created,
        error_status: ErrorStatus::None,
        filepath: Some("test".to_string()),
        patch_id: Some("test".to_string()),
        created_by_id: Some("test".to_string()),
        created_at: Utc::now(),
        updated_by_id: Some("test".to_string()),
        updated_at: Utc::now(),
        device_id: "dvc_123".to_string(),
        config_schema_id: "123".to_string(),
        attempts: 0,
        cooldown_ends_at: Utc::now(),
    };
    let valid_input = json!({
        "id": expected.id,
        "target_status": expected.target_status,
        "activity_status": expected.activity_status,
        "error_status": expected.error_status,
        "filepath": expected.filepath,
        "patch_id": expected.patch_id,
        "created_by_id": expected.created_by_id,
        "created_at": expected.created_at,
        "updated_by_id": expected.updated_by_id,
        "updated_at": expected.updated_at,
        "device_id": expected.device_id,
        "config_schema_id": expected.config_schema_id,
        "attempts": expected.attempts,
        "cooldown_ends_at": expected.cooldown_ends_at,
    });
    let config_instance: ConfigInstance = serde_json::from_value(valid_input).unwrap();
    assert_eq!(config_instance, expected);

    // exclude required fields
    let empty_input = json!({});
    assert!(serde_json::from_value::<ConfigInstance>(empty_input).is_err());

    // exclude default fields
    let expected = ConfigInstance {
        // required fields
        id: "123".to_string(),
        target_status: TargetStatus::Created,
        activity_status: ActivityStatus::Created,
        error_status: ErrorStatus::None,
        device_id: "dvc_123".to_string(),
        config_schema_id: "123".to_string(),
        // rest are defaults
        ..Default::default()
    };
    let valid_input = json!({
        "id": expected.id,
        "target_status": expected.target_status,
        "activity_status": expected.activity_status,
        "error_status": expected.error_status,
        "device_id": expected.device_id,
        "config_schema_id": expected.config_schema_id,
    });
    let config_instance: ConfigInstance = serde_json::from_value(valid_input).unwrap();
    assert_eq!(config_instance, expected);

    // invalid JSON
    assert!(serde_json::from_str::<ConfigInstance>("invalid-json").is_err());
}

#[test]
fn config_instance_from_backend() {
    struct TestCase {
        backend: BackendConfigInstance,
        expected: ConfigInstance,
    }

    let now = Utc::now();

    let test_cases = vec![
        TestCase {
            backend: BackendConfigInstance {
                object: openapi_client::models::backend_config_instance::Object::ConfigInstance,
                id: "cfg_inst_123".to_string(),
                target_status: ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_CREATED,
                status: ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_CREATED,
                activity_status: ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_CREATED,
                error_status: ConfigInstanceErrorStatus::CONFIG_INSTANCE_ERROR_STATUS_NONE,
                filepath: Some("filepath".to_string()),
                patch_id: Some("ptch_123".to_string()),
                created_by_id: Some("created_by_id".to_string()),
                created_at: now.to_rfc3339(),
                updated_by_id: Some("updated_by_id".to_string()),
                updated_at: now.to_rfc3339(),
                device_id: "device_id".to_string(),
                config_schema_id: "config_schema_id".to_string(),
                instance: None,
                created_by: None,
                updated_by: None,
                patch: None,
            },
            expected: ConfigInstance {
                id: "cfg_inst_123".to_string(),
                target_status: TargetStatus::Created,
                activity_status: ActivityStatus::Created,
                error_status: ErrorStatus::None,
                filepath: Some("filepath".to_string()),
                patch_id: Some("ptch_123".to_string()),
                created_by_id: Some("created_by_id".to_string()),
                created_at: now,
                updated_by_id: Some("updated_by_id".to_string()),
                updated_at: now,
                device_id: "device_id".to_string(),
                config_schema_id: "config_schema_id".to_string(),
                attempts: 0,
                cooldown_ends_at: DateTime::<Utc>::UNIX_EPOCH,
            },
        },
    ];

    for test_case in test_cases {
        let config_instance = ConfigInstance::from_backend(test_case.backend.clone());
        assert_eq!(config_instance, test_case.expected);
    }
}

#[test]
fn config_instance_status() {

    struct TestCase {
        instance: ConfigInstance,
        expected: Status,
    }

    let test_cases = vec![
        // activity statuses
        TestCase {
            instance: ConfigInstance {
                activity_status: ActivityStatus::Created,
                error_status: ErrorStatus::None,
                ..Default::default()
            },
            expected: Status::Created,
        },
        TestCase {
            instance: ConfigInstance {
                activity_status: ActivityStatus::Queued,
                error_status: ErrorStatus::None,
                ..Default::default()
            },
            expected: Status::Queued,
        },
        TestCase {
            instance: ConfigInstance {
                activity_status: ActivityStatus::Deployed,
                error_status: ErrorStatus::None,
                ..Default::default()
            },
            expected: Status::Deployed,
        },
        TestCase {
            instance: ConfigInstance {
                activity_status: ActivityStatus::Removed,
                error_status: ErrorStatus::None,
                ..Default::default()
            },
            expected: Status::Removed,
        },
        // error statuses
        TestCase {
            instance: ConfigInstance {
                activity_status: ActivityStatus::Created,
                error_status: ErrorStatus::Retrying,
                ..Default::default()
            },
            expected: Status::Retrying,
        },
        TestCase {
            instance: ConfigInstance {
                activity_status: ActivityStatus::Created,
                error_status: ErrorStatus::Failed,
                ..Default::default()
            },
            expected: Status::Failed,
        },
    ];

    let mut variants = Status::variants().into_iter().collect::<HashSet<_>>();

    for test_case in test_cases {
        variants.remove(&test_case.expected);
        assert_eq!(test_case.instance.status(), test_case.expected);
    }

    assert!(variants.is_empty(), "variants: {:?}", variants);
}

#[test]
fn config_instance_set_cooldown() {

    let mut instance = ConfigInstance::default();
    instance.set_cooldown(TimeDelta::seconds(10));
    let now = Utc::now();
    assert!(instance.cooldown_ends_at < now + TimeDelta::seconds(10));
    assert!(instance.cooldown_ends_at > now + TimeDelta::seconds(9));
}

#[test]
fn config_instance_clear_cooldown() {
    let mut instance = ConfigInstance::default();
    instance.set_cooldown(TimeDelta::seconds(10));
    instance.clear_cooldown();
    assert_eq!(instance.cooldown_ends_at, DateTime::<Utc>::UNIX_EPOCH);
}

#[test]
fn config_instance_is_cooling_down() {
    let mut instance = ConfigInstance::default();
    instance.set_cooldown(TimeDelta::seconds(10));
    assert!(instance.is_in_cooldown());
    instance.clear_cooldown();
    assert!(!instance.is_in_cooldown());
}



