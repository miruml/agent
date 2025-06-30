use config_agent::http::search::{
    format_search_clause,
    format_search_group,
    join,
    LogicalOperator,
    SearchOperator,
};

pub mod format_search_clause_func {
    use super::*;

    #[test]
    fn test_format_search_clause_equals() {
        let clause = format_search_clause(
            "key",
            SearchOperator::Equals,
            vec!["value"],
            false,
        );
        assert_eq!(clause, "key:value");
    }

    #[test]
    fn test_format_search_clause_contains() {
        let clause = format_search_clause(
            "key",
            SearchOperator::Contains,
            vec!["value"],
            false,
        );
        assert_eq!(clause, "key~value");
    }

    #[test]
    fn test_format_search_clause_not_equals() {
        let clause = format_search_clause(
            "key",
            SearchOperator::Equals,
            vec!["value"],
            true,
        );
        assert_eq!(clause, "-key:value");
    }

    #[test]
    fn test_format_search_clause_not_contains() {
        let clause = format_search_clause(
            "key",
            SearchOperator::Contains,
            vec!["value"],
            true,
        );
        assert_eq!(clause, "-key~value");
    }

    #[test]
    fn test_format_search_clause_equals_multiple_values() {
        let clause = format_search_clause(
            "key",
            SearchOperator::Equals,
            vec!["value1", "value2"],
            false,
        );
        assert_eq!(clause, "key:value1,value2");
    }

    #[test]
    fn test_format_search_clause_contains_multiple_values() {
        let clause = format_search_clause(
            "key",
            SearchOperator::Contains,
            vec!["value1", "value2"],
            false,
        );
        assert_eq!(clause, "key~value1,value2");
    }
}

pub mod format_search_group_func {
    use super::*;

    #[test]
    fn test_format_search_group_empty() {
        let group = format_search_group(
            vec![] as Vec<String>,
            LogicalOperator::And,
        );
        assert_eq!(group, None);
    }

    #[test]
    fn test_format_search_group_single_value() {
        let group = format_search_group(
            vec!["key:value"],
            LogicalOperator::And,
        );
        assert_eq!(group, Some("key:value".to_string()));
    }

    #[test]
    fn test_format_search_group_multiple_values() {
        let group = format_search_group(
            vec!["key:value", "key~value"],
            LogicalOperator::And,
        );
        assert_eq!(group, Some("key:value AND key~value".to_string()));
    }
}

pub mod join_func {
    use super::*;

    #[test]
    fn test_join_empty() {
        let joined = join(vec![] as Vec<String>, " AND ");
        assert_eq!(joined, "");
    }

    #[test]
    fn test_join_single_value() {
        let joined = join(vec!["key:value"], " AND ");
        assert_eq!(joined, "key:value");
    }

    #[test]
    fn test_join_multiple_values() {
        let joined = join(vec!["key:value", "key~value"], " AND ");
        assert_eq!(joined, "key:value AND key~value");
    }
}