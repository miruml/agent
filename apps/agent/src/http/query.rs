use crate::http::pagination::Pagination;


pub fn build_query_params(
    search_query: Option<&str>,
    expand_query: Option<&str>,
    pagination: &Pagination,
) -> String {
    let mut query_params = format!(
        "?limit={}&offset={}",
        pagination.limit, pagination.offset,
    );
    if let Some(search_query) = search_query {
        query_params.push_str(&format!("&{}", search_query));
    }
    if let Some(expand_query) = expand_query {
        query_params.push_str(&format!("&{}", expand_query));
    }
    query_params
}
