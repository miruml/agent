use config_agent::http::query::build_query_params;
use config_agent::http::pagination::Pagination;


pub mod build_query_params_func {
    use super::*;

    #[tokio::test]
    async fn pagination() {
        let pagination = Pagination { limit: 10, offset: 0 };
        let query_params = build_query_params(
            None,
            None,
            &pagination,
        );
        assert_eq!(query_params, "?limit=10&offset=0");
    }

    #[tokio::test]
    async fn search() {
        let pagination = Pagination { limit: 10, offset: 0 };
        let search_query = Some("search=device_id:dvc_123");
        let query_params = build_query_params(
            search_query,
            None,
            &pagination,
        );
        assert_eq!(query_params, "?limit=10&offset=0&search=device_id:dvc_123");
    }

    #[tokio::test]
    async fn expand() {
        let pagination = Pagination { limit: 10, offset: 0 };
        let expand_query = Some("expand=device");
        let query_params = build_query_params(
            None,
            expand_query,
            &pagination,
        );
        assert_eq!(query_params, "?limit=10&offset=0&expand=device");
    }
}
