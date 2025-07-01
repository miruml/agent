// standard library
use std::fmt;
use std::sync::Arc;

// internal crates
use crate::http::client::HTTPClient;
use crate::http::errors::HTTPErr;
use crate::http::expand::format_expand_query;
use crate::http::pagination::{MAX_PAGINATE_LIMIT, Pagination};
use crate::http::query::build_query_params;
use crate::http::search::{
    LogicalOperator, SearchOperator, format_search_clause, format_search_group,
};
use openapi_client::models::{
    BackendConfigInstance,
    ConfigInstanceActivityStatus,
    ConfigInstanceErrorStatus,
    ConfigInstanceList,
    ConfigInstanceSearch,
    ConfigInstanceTargetStatus,
    UpdateConfigInstanceRequest,
};

#[allow(async_fn_in_trait)]
pub trait ConfigInstancesExt: Send + Sync {
    async fn list_config_instances(
        &self,
        query_params: &str,
        token: &str,
    ) -> Result<ConfigInstanceList, HTTPErr>;

    async fn list_all_config_instances<I>(
        &self,
        filters: ConfigInstanceFilters,
        expansions: I,
        token: &str,
    ) -> Result<Vec<BackendConfigInstance>, HTTPErr>
    where
        I: IntoIterator + Send,
        I::Item: fmt::Display,
    ;

    async fn update_config_instance(
        &self,
        config_instance_id: &str,
        updates: &UpdateConfigInstanceRequest,
        token: &str,
    ) -> Result<BackendConfigInstance, HTTPErr>;
}

impl HTTPClient {
    fn config_instances_url(&self) -> String {
        format!("{}/config_instances", self.base_url)
    }

    fn config_instance_url(&self, config_instance_id: &str) -> String {
        format!("{}/{}", self.config_instances_url(), config_instance_id)
    }
}

impl ConfigInstancesExt for HTTPClient {
    async fn list_config_instances(
        &self,
        query_params: &str,
        token: &str,
    ) -> Result<ConfigInstanceList, HTTPErr> {

        // build the request
        let url = format!(
            "{}{}",
            self.config_instances_url(),
            query_params,
        );
        let (request, context) = self.build_get_request(
            &url, self.default_timeout, Some(token),
        )?;

        // send the request (with caching)
        let response = self.send_cached(url, request, &context).await?.0;

        // parse the response
        self.parse_json_response_text::<ConfigInstanceList>(response, &context).await
    }

    async fn list_all_config_instances<I>(
        &self,
        filters: ConfigInstanceFilters,
        expansions: I,
        token: &str,
    ) -> Result<Vec<BackendConfigInstance>, HTTPErr>
    where
        I: IntoIterator + Send,
        I::Item: fmt::Display,
    {
        let search_query = build_search_query(filters);
        let expand_query = format_expand_query(expansions);
        let mut pagination = Pagination {
            limit: MAX_PAGINATE_LIMIT, offset: 0,
        };
        let mut config_instances = Vec::new();

        loop {
            let query_params = build_query_params(
                search_query.as_deref(), expand_query.as_deref(), &pagination,
            );
            let resp = self.list_config_instances(&query_params, token).await?;
            config_instances.extend(resp.data);
            if !resp.has_more {
                break;
            }
            pagination.offset += pagination.limit;
        }
        Ok(config_instances)
    }

    async fn update_config_instance(
        &self,
        config_instance_id: &str,
        updates: &UpdateConfigInstanceRequest,
        token: &str,
    ) -> Result<BackendConfigInstance, HTTPErr> {

        // build the request
        let (request, context) = self.build_patch_request(
            &self.config_instance_url(config_instance_id),
            self.marshal_json_payload(updates)?,
            self.default_timeout,
            Some(token),
        )?;

        // send the request (no caching)
        let http_resp = self.send(request, &context).await?;
        let text_resp = self.handle_response(http_resp, &context).await?;

        // parse the response
        self.parse_json_response_text::<BackendConfigInstance>(text_resp, &context).await
    }
}

impl ConfigInstancesExt for Arc<HTTPClient> {
    async fn list_config_instances(
        &self,
        query_params: &str,
        token: &str,
    ) -> Result<ConfigInstanceList, HTTPErr> {
        self.as_ref().list_config_instances(
            query_params,
            token,
        )
        .await
    }

    async fn list_all_config_instances<I>(
        &self,
        filters: ConfigInstanceFilters,
        expansions: I,
        token: &str,
    ) -> Result<Vec<BackendConfigInstance>, HTTPErr>
    where
        I: IntoIterator + Send,
        I::Item: fmt::Display,
    {
        self.as_ref().list_all_config_instances(filters, expansions, token).await
    }

    async fn update_config_instance(
        &self,
        config_instance_id: &str,
        updates: &UpdateConfigInstanceRequest,
        token: &str,
    ) -> Result<BackendConfigInstance, HTTPErr> {
        self.as_ref().update_config_instance(
            config_instance_id,
            updates,
            token,
        )
        .await
    }
}

// ================================ SEARCH FILTERS ================================ //
pub struct ConfigInstanceFilters {
    pub device_id: String,
    pub ids: Option<IDFilter>,
    pub config_schema_ids: Option<ConfigSchemaIDFilter>,
    pub target_statuses: Option<TargetStatusFilter>,
    pub activity_statuses: Option<ActivityStatusFilter>,
    pub error_statuses: Option<ErrorStatusFilter>,
}

pub struct ConfigInstanceFiltersBuilder {
    filters: ConfigInstanceFilters,
}

impl ConfigInstanceFiltersBuilder {
    pub fn new(device_id: String) -> Self {
        Self { filters: ConfigInstanceFilters {
            device_id,
            ids: None,
            config_schema_ids: None,
            target_statuses: None,
            activity_statuses: None,
            error_statuses: None,
        } }
    }

    pub fn with_id_filter(mut self, id_filter: IDFilter) -> Self {
        self.filters.ids = Some(id_filter);
        self
    }

    pub fn with_config_schema_id_filter(mut self, config_schema_id_filter: ConfigSchemaIDFilter) -> Self {
        self.filters.config_schema_ids = Some(config_schema_id_filter);
        self
    }

    pub fn with_target_status_filter(mut self, target_status_filter: TargetStatusFilter) -> Self {
        self.filters.target_statuses = Some(target_status_filter);
        self
    }

    pub fn with_activity_status_filter(mut self, activity_status_filter: ActivityStatusFilter) -> Self {
        self.filters.activity_statuses = Some(activity_status_filter);
        self
    }

    pub fn with_error_status_filter(mut self, error_status_filter: ErrorStatusFilter) -> Self {
        self.filters.error_statuses = Some(error_status_filter);
        self
    }

    pub fn build(self) -> ConfigInstanceFilters {
        self.filters
    }
}

pub struct IDFilter {
    pub not: bool,
    pub op: SearchOperator,
    pub val: Vec<String>,
}

pub struct ConfigSchemaIDFilter {
    pub not: bool,
    pub op: SearchOperator,
    pub val: Vec<String>,
}

pub struct TargetStatusFilter {
    pub not: bool,
    pub op: SearchOperator,
    pub val: Vec<ConfigInstanceTargetStatus>,
}

pub struct ActivityStatusFilter {
    pub not: bool,
    pub op: SearchOperator,
    pub val: Vec<ConfigInstanceActivityStatus>,
}

pub struct ErrorStatusFilter {
    pub not: bool,
    pub op: SearchOperator,
    pub val: Vec<ConfigInstanceErrorStatus>,
}

pub fn build_search_query(filters: ConfigInstanceFilters) -> Option<String> {
    // build the search query
    let mut clauses: Vec<String> = Vec::new();
    if let Some(ids) = filters.ids {
        clauses.push(format_search_clause(
            ConfigInstanceSearch::CONFIG_INSTANCE_SEARCH_ID,
            ids.op,
            ids.val,
            ids.not,
        ));
    }
    clauses.push(format_search_clause(
        ConfigInstanceSearch::CONFIG_INSTANCE_SEARCH_DEVICE_ID,
        SearchOperator::Equals,
        [filters.device_id],
        false,
    ));
    if let Some(config_schema_ids) = filters.config_schema_ids {
        clauses.push(format_search_clause(
            ConfigInstanceSearch::CONFIG_INSTANCE_SEARCH_CONFIG_SCHEMA_ID,
            SearchOperator::Equals,
            config_schema_ids.val,
            config_schema_ids.not,
        ));
    }
    if let Some(target_statuses) = filters.target_statuses {
        clauses.push(format_search_clause(
            ConfigInstanceSearch::CONFIG_INSTANCE_SEARCH_TARGET_STATUS,
            SearchOperator::Equals,
            target_statuses.val,
            target_statuses.not,
        ));
    }
    if let Some(activity_statuses) = filters.activity_statuses {
        clauses.push(format_search_clause(
            ConfigInstanceSearch::CONFIG_INSTANCE_SEARCH_ACTIVITY_STATUS,
            SearchOperator::Equals,
            activity_statuses.val,
            activity_statuses.not,
        ));
    }
    if let Some(error_statuses) = filters.error_statuses {
        clauses.push(format_search_clause(
            ConfigInstanceSearch::CONFIG_INSTANCE_SEARCH_ERROR_STATUS,
            SearchOperator::Equals,
            error_statuses.val,
            error_statuses.not,
        ));
    }
    format_search_group(clauses, LogicalOperator::And).map(|s| format!("search={}", s))
}
