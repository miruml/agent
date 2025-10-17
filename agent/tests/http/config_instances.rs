use miru_agent::http::{
    config_instances::{
        build_search_query, ActivityStatusFilter, ConfigInstanceFiltersBuilder,
        ConfigSchemaIDFilter, ErrorStatusFilter, IDFilter, TargetStatusFilter,
    },
    search::SearchOperator,
};

pub mod build_search_query_func {
    use super::*;

    #[tokio::test]
    async fn ids() {
        let builder = ConfigInstanceFiltersBuilder::new("dvc_123".to_string());
        let filters = builder
            .with_id_filter(IDFilter {
                negate: false,
                op: SearchOperator::Equals,
                val: vec!["1".to_string(), "2".to_string(), "3".to_string()],
            })
            .build();
        let query = build_search_query(filters);
        assert_eq!(
            query,
            Some("search=id:1|2|3 AND device_id:dvc_123".to_string())
        );
    }

    #[tokio::test]
    async fn config_schema_ids() {
        let builder = ConfigInstanceFiltersBuilder::new("dvc_123".to_string());
        let filters = builder
            .with_config_schema_id_filter(ConfigSchemaIDFilter {
                negate: false,
                op: SearchOperator::Equals,
                val: vec!["1".to_string(), "2".to_string(), "3".to_string()],
            })
            .build();
        let query = build_search_query(filters);
        assert_eq!(
            query,
            Some("search=device_id:dvc_123 AND config_schema_id:1|2|3".to_string())
        );
    }

    #[tokio::test]
    async fn target_statuses() {
        let builder = ConfigInstanceFiltersBuilder::new("dvc_123".to_string());
        let filters = builder.with_target_status_filter(
            TargetStatusFilter {
                negate: false,
                op: SearchOperator::Equals,
                val: vec![openapi_client::models::ConfigInstanceTargetStatus::CONFIG_INSTANCE_TARGET_STATUS_REMOVED],
            }
        ).build();
        let query = build_search_query(filters);
        assert_eq!(
            query,
            Some("search=device_id:dvc_123 AND target_status:removed".to_string())
        );
    }

    #[tokio::test]
    async fn activity_statuses() {
        let builder = ConfigInstanceFiltersBuilder::new("dvc_123".to_string());
        let filters = builder.with_activity_status_filter(
            ActivityStatusFilter {
                negate: false,
                op: SearchOperator::Equals,
                val: vec![openapi_client::models::ConfigInstanceActivityStatus::CONFIG_INSTANCE_ACTIVITY_STATUS_DEPLOYED],
            }
        ).build();
        let query = build_search_query(filters);
        assert_eq!(
            query,
            Some("search=device_id:dvc_123 AND activity_status:deployed".to_string())
        );
    }

    #[tokio::test]
    async fn error_statuses() {
        let builder = ConfigInstanceFiltersBuilder::new("dvc_123".to_string());
        let filters = builder.with_error_status_filter(
            ErrorStatusFilter {
                negate: false,
                op: SearchOperator::Equals,
                val: vec![openapi_client::models::ConfigInstanceErrorStatus::CONFIG_INSTANCE_ERROR_STATUS_NONE],
            }
        ).build();
        let query = build_search_query(filters);
        assert_eq!(
            query,
            Some("search=device_id:dvc_123 AND error_status:none".to_string())
        );
    }
}
