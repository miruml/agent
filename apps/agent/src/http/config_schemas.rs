// standard library
use std::sync::Arc;

// internal crates
use crate::crypt::sha256;
use crate::http::client::HTTPClient;
use crate::http::errors::HTTPErr;
use crate::http::search::{LogicalOperator, SearchClause, SearchGroup, SearchOperator};
use openapi_client::models::{
    hash_schema_serialized_request::HashSchemaSerializedRequest, ConfigSchemaList,
    ConfigSchemaSearch, SchemaDigestResponse,
};

#[allow(async_fn_in_trait)]
pub trait ConfigSchemasExt: Send + Sync {
    async fn hash_schema(
        &self,
        payload: &HashSchemaSerializedRequest,
        token: &str,
    ) -> Result<SchemaDigestResponse, HTTPErr>;

    async fn list_config_schemas(
        &self,
        digests: &[String],
        config_type_slugs: &[String],
        token: &str,
    ) -> Result<ConfigSchemaList, HTTPErr>;
}

impl HTTPClient {
    fn config_schemas_url(&self) -> String {
        format!("{}/config_schemas", self.base_url)
    }
}

impl ConfigSchemasExt for HTTPClient {
    async fn hash_schema(
        &self,
        payload: &HashSchemaSerializedRequest,
        token: &str,
    ) -> Result<SchemaDigestResponse, HTTPErr> {
        // build the request
        let url = format!("{}/config_schemas/hash/serialized", self.base_url);
        let (request, context) = self.build_post_request(
            &url,
            self.marshal_json_payload(payload)?,
            self.default_timeout,
            Some(token),
        )?;

        // send the request (with caching)
        let key = format!("{}:{}", url, sha256::hash_bytes(&payload.schema));
        let response = self.send_cached(key, request, &context).await?.0;

        // parse the response
        self.parse_json_response_text::<SchemaDigestResponse>(response, &context)
            .await
    }

    async fn list_config_schemas(
        &self,
        digests: &[String],
        config_type_slugs: &[String],
        token: &str,
    ) -> Result<ConfigSchemaList, HTTPErr> {

        // build the search query
        let mut clauses: Vec<SearchClause> = Vec::new();
        if !digests.is_empty() {
            clauses.push(SearchClause {
                key: ConfigSchemaSearch::CONFIG_SCHEMA_SEARCH_DIGEST.to_string(),
                op: SearchOperator::Equals,
                values: digests.iter().map(|s| s.to_string()).collect(),
            });
        }
        if !config_type_slugs.is_empty() {
            clauses.push(SearchClause {
                key: ConfigSchemaSearch::CONFIG_SCHEMA_SEARCH_CONFIG_TYPE_SLUG.to_string(),
                op: SearchOperator::Equals,
                values: config_type_slugs.iter().map(|s| s.to_string()).collect(),
            });
        }

        let query = if clauses.is_empty() {
            "".to_string()
        } else {
            let search_query = SearchGroup {
                clauses,
                op: LogicalOperator::And,
            };
            format!("?search={}", search_query)
        };

        // build the request
        let url = format!("{}{}", self.config_schemas_url(), query);
        let (request, context) = self.build_get_request(&url, self.default_timeout, Some(token))?;

        // send the request (with caching)
        let response = self.send_cached(url, request, &context).await?.0;

        // parse the response
        self.parse_json_response_text::<ConfigSchemaList>(response, &context)
            .await
    }
}

impl ConfigSchemasExt for Arc<HTTPClient> {
    async fn hash_schema(
        &self,
        payload: &HashSchemaSerializedRequest,
        token: &str,
    ) -> Result<SchemaDigestResponse, HTTPErr> {
        self.as_ref().hash_schema(payload, token).await
    }

    async fn list_config_schemas(
        &self,
        digests: &[String],
        config_type_slugs: &[String],
        token: &str,
    ) -> Result<ConfigSchemaList, HTTPErr> {
        self.as_ref().list_config_schemas(digests, config_type_slugs, token).await
    }
}