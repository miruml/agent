// standard library
use std::collections::HashSet;

// internal crates
use config_agent::models::config_instance::{
    ActivityStatus, ConfigInstance, ErrorStatus, Status, TargetStatus,
};
use openapi_client::models::{
    ConfigInstance as BackendConfigInstance, ConfigInstanceActivityStatus,
    ConfigInstanceErrorStatus, ConfigInstanceStatus, ConfigInstanceTargetStatus,
};

// external crates
use chrono::{DateTime, TimeDelta, Utc};
use serde_json::json;
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

#[test]
fn serialize_deserialize_target_status() {
    struct TestCase {
        input: &'static str,
        expected: TargetStatus,
        valid: bool,
    }

    let test_cases = vec![
        // valid
        TestCase {
            input: "\"created\"",
            expected: TargetStatus::Created,
            valid: true,
        },
        TestCase {
            input: "\"validated\"",
            expected: TargetStatus::Validated,
            valid: true,
        },
        TestCase {
            input: "\"deployed\"",
            expected: TargetStatus::Deployed,
            valid: true,
        },
        TestCase {
            input: "\"removed\"",
            expected: TargetStatus::Removed,
            valid: true,
        },
        // default
        TestCase {
            input: "\"unknown\"",
            expected: TargetStatus::Created,
            valid: false,
        },
    ];

    let mut variants = TargetStatus::variants().into_iter().collect::<HashSet<_>>();

    for test_case in test_cases {
        variants.remove(&test_case.expected);
        let deserialized = serde_json::from_str::<TargetStatus>(test_case.input).unwrap();
        assert_eq!(deserialized, test_case.expected);
        if test_case.valid {
            let serialized = serde_json::to_string(&test_case.expected).unwrap();
            assert_eq!(serialized, test_case.input);
        }
    }

    assert!(variants.is_empty(), "variants: {variants:?}");
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
            storage: TargetStatus::Validated,
            backend: openapi_client::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_VALIDATED,
            sdk: openapi_server::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_VALIDATED,
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
        assert_eq!(
            TargetStatus::from_backend(&test_case.backend),
            test_case.storage
        );
        assert_eq!(
            test_case.backend,
            TargetStatus::to_backend(&test_case.storage)
        );
        assert_eq!(TargetStatus::from_sdk(&test_case.sdk), test_case.storage);
        assert_eq!(test_case.sdk, TargetStatus::to_sdk(&test_case.storage));
    }

    assert!(variants.is_empty(), "variants: {variants:?}");
}

#[test]
fn serialize_deserialize_activity_status() {
    struct TestCase {
        input: &'static str,
        expected: ActivityStatus,
        valid: bool,
    }

    let test_cases = vec![
        // valid
        TestCase {
            input: "\"created\"",
            expected: ActivityStatus::Created,
            valid: true,
        },
        TestCase {
            input: "\"validating\"",
            expected: ActivityStatus::Validating,
            valid: true,
        },
        TestCase {
            input: "\"validated\"",
            expected: ActivityStatus::Validated,
            valid: true,
        },
        TestCase {
            input: "\"queued\"",
            expected: ActivityStatus::Queued,
            valid: true,
        },
        TestCase {
            input: "\"deployed\"",
            expected: ActivityStatus::Deployed,
            valid: true,
        },
        TestCase {
            input: "\"removed\"",
            expected: ActivityStatus::Removed,
            valid: true,
        },
        TestCase {
            input: "\"unknown\"",
            expected: ActivityStatus::Created,
            valid: false,
        },
    ];

    let mut variants = ActivityStatus::variants()
        .into_iter()
        .collect::<HashSet<_>>();

    for test_case in test_cases {
        variants.remove(&test_case.expected);
        let deserialized = serde_json::from_str::<ActivityStatus>(test_case.input).unwrap();
        assert_eq!(deserialized, test_case.expected);
        if test_case.valid {
            let serialized = serde_json::to_string(&test_case.expected).unwrap();
            assert_eq!(serialized, test_case.input);
        }
    }

    assert!(variants.is_empty(), "variants: {variants:?}");
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
            storage: ActivityStatus::Validating,
            backend: openapi_client::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_VALIDATING,
            sdk: openapi_server::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_VALIDATING,
        },
        TestCase {
            storage: ActivityStatus::Validated,
            backend: openapi_client::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_VALIDATED,
            sdk: openapi_server::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_VALIDATED,
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

    let mut variants = ActivityStatus::variants()
        .into_iter()
        .collect::<HashSet<_>>();

    for test_case in test_cases {
        variants.remove(&test_case.storage);
        assert_eq!(
            ActivityStatus::from_backend(&test_case.backend),
            test_case.storage
        );
        assert_eq!(
            test_case.backend,
            ActivityStatus::to_backend(&test_case.storage)
        );
        assert_eq!(ActivityStatus::from_sdk(&test_case.sdk), test_case.storage);
        assert_eq!(test_case.sdk, ActivityStatus::to_sdk(&test_case.storage));
    }

    assert!(variants.is_empty(), "variants: {variants:?}");
}

#[test]
fn serialize_deserialize_error_status() {
    struct TestCase {
        input: &'static str,
        expected: ErrorStatus,
        valid: bool,
    }

    let test_cases = vec![
        // valid
        TestCase {
            input: "\"none\"",
            expected: ErrorStatus::None,
            valid: true,
        },
        TestCase {
            input: "\"failed\"",
            expected: ErrorStatus::Failed,
            valid: true,
        },
        TestCase {
            input: "\"retrying\"",
            expected: ErrorStatus::Retrying,
            valid: true,
        },
        // default
        TestCase {
            input: "\"unknown\"",
            expected: ErrorStatus::None,
            valid: false,
        },
    ];

    let mut variants = ErrorStatus::variants().into_iter().collect::<HashSet<_>>();

    for test_case in test_cases {
        variants.remove(&test_case.expected);
        let deserialized = serde_json::from_str::<ErrorStatus>(test_case.input).unwrap();
        assert_eq!(deserialized, test_case.expected);
        if test_case.valid {
            let serialized = serde_json::to_string(&test_case.expected).unwrap();
            assert_eq!(serialized, test_case.input);
        }
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
        assert_eq!(
            ErrorStatus::from_backend(&test_case.backend),
            test_case.storage
        );
        assert_eq!(
            test_case.backend,
            ErrorStatus::to_backend(&test_case.storage)
        );
        assert_eq!(ErrorStatus::from_sdk(&test_case.sdk), test_case.storage);
        assert_eq!(test_case.sdk, ErrorStatus::to_sdk(&test_case.storage));
    }

    assert!(variants.is_empty(), "variants: {variants:?}");
}

#[test]
fn serialize_deserialize_status() {
    struct TestCase {
        input: &'static str,
        expected: Status,
        valid: bool,
    }

    let test_cases = vec![
        // valid
        TestCase {
            input: "\"created\"",
            expected: Status::Created,
            valid: true,
        },
        TestCase {
            input: "\"queued\"",
            expected: Status::Queued,
            valid: true,
        },
        TestCase {
            input: "\"validating\"",
            expected: Status::Validating,
            valid: true,
        },
        TestCase {
            input: "\"validated\"",
            expected: Status::Validated,
            valid: true,
        },
        TestCase {
            input: "\"deployed\"",
            expected: Status::Deployed,
            valid: true,
        },
        TestCase {
            input: "\"removed\"",
            expected: Status::Removed,
            valid: true,
        },
        TestCase {
            input: "\"failed\"",
            expected: Status::Failed,
            valid: true,
        },
        TestCase {
            input: "\"retrying\"",
            expected: Status::Retrying,
            valid: true,
        },
        // default
        TestCase {
            input: "\"unknown\"",
            expected: Status::Created,
            valid: false,
        },
    ];

    let mut variants = Status::variants().into_iter().collect::<HashSet<_>>();

    for test_case in test_cases {
        variants.remove(&test_case.expected);
        let deserialized = serde_json::from_str::<Status>(test_case.input).unwrap();
        assert_eq!(deserialized, test_case.expected);
        if test_case.valid {
            let serialized = serde_json::to_string(&test_case.expected).unwrap();
            assert_eq!(test_case.input, serialized);
        }
    }

    assert!(variants.is_empty(), "variants: {variants:?}");
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
            storage: Status::Validating,
            backend:
                openapi_client::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_VALIDATING,
            sdk: openapi_server::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_VALIDATING,
        },
        TestCase {
            storage: Status::Validated,
            backend: openapi_client::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_VALIDATED,
            sdk: openapi_server::models::ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_VALIDATED,
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

    assert!(variants.is_empty(), "variants: {variants:?}");
}

#[test]
fn serialize_deserialize_config_instance() {
    let expected = ConfigInstance {
        id: "123".to_string(),
        target_status: TargetStatus::Removed,
        activity_status: ActivityStatus::Removed,
        error_status: ErrorStatus::Failed,
        relative_filepath: "test".to_string(),
        patch_id: Some("test".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        device_id: "dvc_123".to_string(),
        config_schema_id: "123".to_string(),
        config_type_id: "123".to_string(),
        attempts: 0,
        cooldown_ends_at: Utc::now(),
    };
    let serialized = serde_json::to_string(&expected).unwrap();
    let deserialized = serde_json::from_str::<ConfigInstance>(&serialized).unwrap();
    assert_eq!(deserialized, expected);
}

#[tokio::test]
async fn deserialize_config_instance() {
    // valid deserialization
    let expected = ConfigInstance {
        id: "123".to_string(),
        target_status: TargetStatus::Created,
        activity_status: ActivityStatus::Created,
        error_status: ErrorStatus::None,
        relative_filepath: "test".to_string(),
        patch_id: Some("test".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        device_id: "dvc_123".to_string(),
        config_schema_id: "123".to_string(),
        config_type_id: "123".to_string(),
        attempts: 0,
        cooldown_ends_at: Utc::now(),
    };
    let valid_input = json!({
        "id": expected.id,
        "target_status": expected.target_status,
        "activity_status": expected.activity_status,
        "error_status": expected.error_status,
        "relative_filepath": expected.relative_filepath,
        "patch_id": expected.patch_id,
        "created_at": expected.created_at,
        "updated_at": expected.updated_at,
        "device_id": expected.device_id,
        "config_schema_id": expected.config_schema_id,
        "config_type_id": expected.config_type_id,
        "attempts": expected.attempts,
        "cooldown_ends_at": expected.cooldown_ends_at,
    });
    let config_instance: ConfigInstance = serde_json::from_value(valid_input).unwrap();
    info!("config_instance: {:?}", config_instance);
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
        config_type_id: "123".to_string(),
        // rest are defaults
        ..Default::default()
    };
    let valid_input = json!({
        "id": expected.id,
        "target_status": expected.target_status,
        "activity_status": expected.activity_status,
        "relative_filepath": expected.relative_filepath,
        "error_status": expected.error_status,
        "device_id": expected.device_id,
        "config_schema_id": expected.config_schema_id,
        "config_type_id": expected.config_type_id,
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

    let test_cases = vec![TestCase {
        backend: BackendConfigInstance {
            object: openapi_client::models::config_instance::Object::ConfigInstance,
            id: "cfg_inst_123".to_string(),
            target_status: ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_CREATED,
            status: ConfigInstanceStatus::CONFIG_INSTANCE_STATUS_CREATED,
            activity_status: ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_CREATED,
            error_status: ConfigInstanceErrorStatus::CONFIG_INSTANCE_ERROR_STATUS_NONE,
            relative_filepath: "filepath".to_string(),
            patch_id: Some("ptch_123".to_string()),
            created_by_id: "created_by_id".to_string(),
            created_at: now.to_rfc3339(),
            updated_by_id: "updated_by_id".to_string(),
            updated_at: now.to_rfc3339(),
            device_id: "device_id".to_string(),
            config_schema_id: "config_schema_id".to_string(),
            config_type_id: "config_type_id".to_string(),
            config_type: None,
            content: None,
            created_by: None,
            updated_by: None,
            patch: None,
            config_schema: None,
            device: None,
        },
        expected: ConfigInstance {
            id: "cfg_inst_123".to_string(),
            target_status: TargetStatus::Created,
            activity_status: ActivityStatus::Created,
            error_status: ErrorStatus::None,
            relative_filepath: "filepath".to_string(),
            patch_id: Some("ptch_123".to_string()),
            created_at: now,
            updated_at: now,
            device_id: "device_id".to_string(),
            config_schema_id: "config_schema_id".to_string(),
            config_type_id: "config_type_id".to_string(),
            attempts: 0,
            cooldown_ends_at: DateTime::<Utc>::UNIX_EPOCH,
        },
    }];

    for test_case in test_cases {
        let config_instance = ConfigInstance::from_backend(test_case.backend.clone());
        assert_eq!(config_instance, test_case.expected);
    }
}

#[test]
fn config_instance_status() {
    struct TestCase {
        cfg_inst: ConfigInstance,
        expected: Status,
    }

    let test_cases = vec![
        // activity statuses
        TestCase {
            cfg_inst: ConfigInstance {
                activity_status: ActivityStatus::Created,
                error_status: ErrorStatus::None,
                ..Default::default()
            },
            expected: Status::Created,
        },
        TestCase {
            cfg_inst: ConfigInstance {
                activity_status: ActivityStatus::Validating,
                error_status: ErrorStatus::None,
                ..Default::default()
            },
            expected: Status::Validating,
        },
        TestCase {
            cfg_inst: ConfigInstance {
                activity_status: ActivityStatus::Validated,
                error_status: ErrorStatus::None,
                ..Default::default()
            },
            expected: Status::Validated,
        },
        TestCase {
            cfg_inst: ConfigInstance {
                activity_status: ActivityStatus::Queued,
                error_status: ErrorStatus::None,
                ..Default::default()
            },
            expected: Status::Queued,
        },
        TestCase {
            cfg_inst: ConfigInstance {
                activity_status: ActivityStatus::Deployed,
                error_status: ErrorStatus::None,
                ..Default::default()
            },
            expected: Status::Deployed,
        },
        TestCase {
            cfg_inst: ConfigInstance {
                activity_status: ActivityStatus::Removed,
                error_status: ErrorStatus::None,
                ..Default::default()
            },
            expected: Status::Removed,
        },
        // error statuses
        TestCase {
            cfg_inst: ConfigInstance {
                activity_status: ActivityStatus::Created,
                error_status: ErrorStatus::Retrying,
                ..Default::default()
            },
            expected: Status::Retrying,
        },
        TestCase {
            cfg_inst: ConfigInstance {
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
        assert_eq!(test_case.cfg_inst.status(), test_case.expected);
    }

    assert!(variants.is_empty(), "variants: {variants:?}");
}

#[test]
fn config_instance_set_cooldown() {
    let mut cfg_inst = ConfigInstance::default();
    cfg_inst.set_cooldown(TimeDelta::seconds(10));
    let now = Utc::now();
    assert!(cfg_inst.cooldown_ends_at < now + TimeDelta::seconds(10));
    assert!(cfg_inst.cooldown_ends_at > now + TimeDelta::seconds(9));
}

#[test]
fn config_instance_clear_cooldown() {
    let mut cfg_inst = ConfigInstance::default();
    cfg_inst.set_cooldown(TimeDelta::seconds(10));
    cfg_inst.clear_cooldown();
    assert_eq!(cfg_inst.cooldown_ends_at, DateTime::<Utc>::UNIX_EPOCH);
}

#[test]
fn config_instance_is_cooling_down() {
    let mut cfg_inst = ConfigInstance::default();
    cfg_inst.set_cooldown(TimeDelta::seconds(10));
    assert!(cfg_inst.is_in_cooldown());
    cfg_inst.clear_cooldown();
    assert!(!cfg_inst.is_in_cooldown());
}

#[test]
fn config_instance_cooldown() {
    let mut cfg_inst = ConfigInstance::default();
    let cooldown = TimeDelta::seconds(10);
    cfg_inst.set_cooldown(cooldown);
    assert!(cfg_inst.cooldown() < cooldown);
    assert!(cfg_inst.cooldown() > cooldown - TimeDelta::seconds(1));
}
