// internal crates
use miru_agent::http::expand::format_expand_query;

pub mod format_expand_query {
    use super::*;

    #[tokio::test]
    async fn test_format_expand_query_exists() {
        // one query
        let expand_query = format_expand_query(vec!["test"]).unwrap();
        assert_eq!(expand_query, "expand[]=test");

        // two queries
        let expand_query = format_expand_query(vec!["test", "test2"]).unwrap();
        assert_eq!(expand_query, "expand[]=test&expand[]=test2");

        // four queries
        let expand_query = format_expand_query(vec!["test", "test2", "test3", "test4"]).unwrap();
        assert_eq!(
            expand_query,
            "expand[]=test&expand[]=test2&expand[]=test3&expand[]=test4"
        );
    }

    #[tokio::test]
    async fn test_format_expand_query_empty() {
        let expand_query = format_expand_query(vec![] as Vec<&str>);
        assert!(expand_query.is_none());
    }
}
