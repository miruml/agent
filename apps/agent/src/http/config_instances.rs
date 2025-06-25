// standard library
use std::sync::Arc;

// internal crates
use crate::http::client::HTTPClient;
use crate::http::errors::HTTPErr;
use crate::http::search::{LogicalOperator, SearchClause, SearchGroup, SearchOperator};
use openapi_client::models::{
    BackendConfigInstance, ConfigInstanceActivityStatus, ConfigInstanceErrorStatus,
    ConfigInstanceList, ConfigInstanceSearch, ConfigInstanceTargetStatus,
    RefreshLatestConfigInstanceRequest,
};

#[allow(async_fn_in_trait)]
pub trait ConfigInstancesExt: Send + Sync {
    async fn read_latest_config_instance(
        &self,
        device_id: &str,
        config_type_slug: &str,
        config_schema_digest: &str,
        token: &str,
    ) -> Result<Option<BackendConfigInstance>, HTTPErr>;

    async fn refresh_latest_config_instance(
        &self,
        payload: &RefreshLatestConfigInstanceRequest,
        token: &str,
    ) -> Result<BackendConfigInstance, HTTPErr>;
}

impl HTTPClient {
    fn config_instances_url(&self) -> String {
        format!("{}/config_instances", self.base_url)
    }

    async fn list_config_instances(
        &self,
        device_id: String,
        config_schema_ids: &[String],
        target_statuses: &[ConfigInstanceTargetStatus],
        activity_statuses: &[ConfigInstanceActivityStatus],
        error_statuses: &[ConfigInstanceErrorStatus],
        token: &str,
    ) -> Result<ConfigInstanceList, HTTPErr> {

        // build the search query
        let mut clauses: Vec<SearchClause> = Vec::new();
        clauses.push(SearchClause {
            key: ConfigInstanceSearch::CONFIG_INSTANCE_SEARCH_DEVICE_ID.to_string(),
            op: SearchOperator::Equals,
            values: vec![device_id],
        });
        if !config_schema_ids.is_empty() {
            clauses.push(SearchClause {
                key: ConfigInstanceSearch::CONFIG_INSTANCE_SEARCH_CONFIG_SCHEMA_ID.to_string(),
                op: SearchOperator::Equals,
                values: config_schema_ids.iter().map(|s| s.to_string()).collect(),
            });
        }
        if !target_statuses.is_empty() {
            clauses.push(SearchClause {
                key: ConfigInstanceSearch::CONFIG_INSTANCE_SEARCH_TARGET_STATUS.to_string(),
                op: SearchOperator::Equals,
                values: target_statuses.iter().map(|s| s.to_string()).collect(),
            });
        }
        if !activity_statuses.is_empty() {
            clauses.push(SearchClause {
                key: ConfigInstanceSearch::CONFIG_INSTANCE_SEARCH_ACTIVITY_STATUS.to_string(),
                op: SearchOperator::Equals,
                values: activity_statuses.iter().map(|s| s.to_string()).collect(),
            });
        }
        if !error_statuses.is_empty() {
            clauses.push(SearchClause {
                key: ConfigInstanceSearch::CONFIG_INSTANCE_SEARCH_ERROR_STATUS.to_string(),
                op: SearchOperator::Equals,
                values: error_statuses.iter().map(|s| s.to_string()).collect(),
            });
        }
        let search_query = SearchGroup {
            clauses,
            op: LogicalOperator::And,
        };

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

impl ConfigInstancesExt for HTTPClient {
    async fn read_latest_config_instance(
        &self,
        device_id: &str,
        config_type_slug: &str,
        config_schema_digest: &str,
        token: &str,
    ) -> Result<Option<BackendConfigInstance>, HTTPErr> {
        // build the request
        let url = format!(
            "{}/latest?device_id={}&config_type_slug={}&config_schema_digest={}",
            self.config_instances_url(),
            device_id,
            config_type_slug,
            config_schema_digest
        );
        let (request, context) = self.build_get_request(&url, self.default_timeout, Some(token))?;

        // send the request (with caching)
        let response = self.send_cached(url, request, &context).await?.0;

        // parse the response
        self.parse_json_response_text::<Option<BackendConfigInstance>>(response, &context)
            .await
    }

    async fn refresh_latest_config_instance(
        &self,
        payload: &RefreshLatestConfigInstanceRequest,
        token: &str,
    ) -> Result<BackendConfigInstance, HTTPErr> {
        // build the request
        let url = format!("{}/refresh_latest", self.config_instances_url());
        let key = format!(
            "{}:{}:{}",
            url, payload.config_type_slug, payload.config_schema_digest,
        );
        let (request, context) = self.build_post_request(
            &url,
            self.marshal_json_payload(payload)?,
            self.default_timeout,
            Some(token),
        )?;

        // send the request
        let response = self.send_cached(key, request, &context).await?.0;

        // parse the response
        self.parse_json_response_text::<BackendConfigInstance>(response, &context)
            .await
    }
}

impl ConfigInstancesExt for Arc<HTTPClient> {
    async fn read_latest_config_instance(
        &self,
        device_id: &str,
        config_type_slug: &str,
        config_schema_digest: &str,
        token: &str,
    ) -> Result<Option<BackendConfigInstance>, HTTPErr> {
        self.as_ref()
            .read_latest_config_instance(device_id, config_type_slug, config_schema_digest, token)
            .await
    }

    async fn refresh_latest_config_instance(
        &self,
        request: &RefreshLatestConfigInstanceRequest,
        token: &str,
    ) -> Result<BackendConfigInstance, HTTPErr> {
        self.as_ref()
            .refresh_latest_config_instance(request, token)
            .await
    }
}
