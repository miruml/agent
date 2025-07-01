use config_agent::http::{
    config_schemas::{
        ConfigSchemaFilters,
        ConfigTypeSlugFilter,
        DigestFilter,
        build_search_query,
    },
    search::SearchOperator,
};


pub mod build_search_query_func {
    use super::*;

    #[tokio::test]
    async fn empty() {
        let filters = ConfigSchemaFilters {
            digests: None,
            config_type_slugs: None,
        };
        let query = build_search_query(filters);
        assert_eq!(query, None);
    }

    #[tokio::test]
    async fn config_type_slug() {
        let filters = ConfigSchemaFilters {
            digests: None,
            config_type_slugs: Some(ConfigTypeSlugFilter {
                not: false,
                op: SearchOperator::Equals,
                val: vec!["1".to_string(), "2".to_string(), "3".to_string()],
            }),
        };
        let query = build_search_query(filters);
        assert_eq!(query, Some("search=config_type_slug:1,2,3".to_string()));
    }

    #[tokio::test]
    async fn digest() {
        let filters = ConfigSchemaFilters {
            digests: Some(DigestFilter {
                not: false,
                op: SearchOperator::Equals,
                val: vec!["1".to_string(), "2".to_string(), "3".to_string()],
            }),
            config_type_slugs: None,
        };
        let query = build_search_query(filters);
        assert_eq!(query, Some("search=digest:1,2,3".to_string()));
    }
}
