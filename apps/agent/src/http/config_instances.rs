// standard library
use std::fmt;
use std::sync::Arc;

// internal crates
use crate::http::client::HTTPClient;
use crate::http::errors::HTTPErr;
use crate::http::search::{
    LogicalOperator, SearchOperator, format_search_clause, format_search_group,
};
use openapi_client::models::{
    ConfigInstanceActivityStatus,
    ConfigInstanceErrorStatus,
    ConfigInstanceList,
    ConfigInstanceSearch,
    ConfigInstanceTargetStatus,
};

#[allow(async_fn_in_trait)]
pub trait ConfigInstancesExt: Send + Sync {
    async fn list_config_instances<
        S1, S2, S3, S4
    >(
        &self,
        device_id: &str,
        config_schema_ids: S1,
        target_statuses: S2,
        activity_statuses: S3,
        error_statuses: S4,
        token: &str,
    ) -> Result<ConfigInstanceList, HTTPErr>
    where
        S1: IntoIterator,
        S1::Item: fmt::Display,
        S2: IntoIterator<Item = ConfigInstanceTargetStatus>,
        S3: IntoIterator<Item = ConfigInstanceActivityStatus>,
        S4: IntoIterator<Item = ConfigInstanceErrorStatus>;
}

impl HTTPClient {
    fn config_instances_url(&self) -> String {
        format!("{}/config_instances", self.base_url)
    }
}

impl ConfigInstancesExt for HTTPClient {
    async fn list_config_instances<
        S1, S2, S3, S4,
    >(
        &self,
        device_id: &str,
        config_schema_ids: S1,
        target_statuses: S2,
        activity_statuses: S3,
        error_statuses: S4,
        token: &str,
    ) -> Result<ConfigInstanceList, HTTPErr>
    where
        S1: IntoIterator,
        S1::Item: fmt::Display,
        S2: IntoIterator<Item = ConfigInstanceTargetStatus>,
        S3: IntoIterator<Item = ConfigInstanceActivityStatus>,
        S4: IntoIterator<Item = ConfigInstanceErrorStatus>,
    {
        // build the search query
        let mut clauses: Vec<String> = Vec::new();
        clauses.push(format_search_clause(
            ConfigInstanceSearch::CONFIG_INSTANCE_SEARCH_DEVICE_ID,
            SearchOperator::Equals,
            [device_id],
        ));
        let mut config_schema_ids_iter = config_schema_ids.into_iter().peekable();
        if config_schema_ids_iter.peek().is_some() {
            clauses.push(format_search_clause(
                ConfigInstanceSearch::CONFIG_INSTANCE_SEARCH_CONFIG_SCHEMA_ID,
                SearchOperator::Equals,
                config_schema_ids_iter,
            ));
        }
        let mut target_status_iter = target_statuses.into_iter().peekable();
        if target_status_iter.peek().is_some() {
            clauses.push(format_search_clause(
                ConfigInstanceSearch::CONFIG_INSTANCE_SEARCH_TARGET_STATUS,
                SearchOperator::Equals,
                target_status_iter,
            ));
        }
        let mut activity_statuses_iter = activity_statuses.into_iter().peekable();
        if activity_statuses_iter.peek().is_some() {
            clauses.push(format_search_clause(
                ConfigInstanceSearch::CONFIG_INSTANCE_SEARCH_ACTIVITY_STATUS,
                SearchOperator::Equals,
                activity_statuses_iter,
            ));
        }
        let mut error_statuses_iter = error_statuses.into_iter().peekable();
        if error_statuses_iter.peek().is_some() {
            clauses.push(format_search_clause(
                ConfigInstanceSearch::CONFIG_INSTANCE_SEARCH_ERROR_STATUS,
                SearchOperator::Equals,
                error_statuses_iter,
            ));
        }
        let search_query = format_search_group(clauses, LogicalOperator::And);

        // build the request
        let url = format!("{}?search={}", self.config_instances_url(), search_query,);
        let (request, context) = self.build_get_request(&url, self.default_timeout, Some(token))?;

        // send the request (with caching)
        let response = self.send_cached(url, request, &context).await?.0;

        // parse the response
        self.parse_json_response_text::<ConfigInstanceList>(response, &context)
            .await
    }
}

impl ConfigInstancesExt for Arc<HTTPClient> {
    async fn list_config_instances<
        S1, S2, S3, S4,
    >(
        &self,
        device_id: &str,
        config_schema_ids: S1,
        target_statuses: S2,
        activity_statuses: S3,
        error_statuses: S4,
        token: &str,
    ) -> Result<ConfigInstanceList, HTTPErr>
    where
        S1: IntoIterator,
        S1::Item: fmt::Display,
        S2: IntoIterator<Item = ConfigInstanceTargetStatus>,
        S3: IntoIterator<Item = ConfigInstanceActivityStatus>,
        S4: IntoIterator<Item = ConfigInstanceErrorStatus>,
    {
        self.as_ref().list_config_instances(
            device_id,
            config_schema_ids,
            target_statuses,
            activity_statuses,
            error_statuses,
            token,
        )
        .await
    }
}
