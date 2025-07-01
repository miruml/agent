// standard library
use std::sync::Arc;

// internal crates
use crate::crypt::sha256;
use crate::http::client::HTTPClient;
use crate::http::errors::HTTPErr;
use crate::http::pagination::Pagination;
use crate::http::query::build_query_params;
use crate::http::search::{
    LogicalOperator, SearchOperator, format_search_clause, format_search_group,
};
use crate::http::errors::{TooManyConfigSchemas, ConfigSchemaNotFound};
use openapi_client::models::{
    hash_schema_serialized_request::HashSchemaSerializedRequest, ConfigSchema, ConfigSchemaList,
    ConfigSchemaSearch, SchemaDigestResponse,
};
use crate::trace;

#[allow(async_fn_in_trait)]
pub trait ConfigSchemasExt: Send + Sync {
    async fn hash_schema(
        &self,
        payload: &HashSchemaSerializedRequest,
        token: &str,
    ) -> Result<SchemaDigestResponse, HTTPErr>;

    async fn list_config_schemas(
        &self,
        query_params: &str,
        token: &str,
    ) -> Result<ConfigSchemaList, HTTPErr>;

    async fn find_one_config_schema(
        &self,
        filters: ConfigSchemaFilters,
        token: &str,
    ) -> Result<ConfigSchema, HTTPErr>;
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

    async fn find_one_config_schema(
        &self,
        filters: ConfigSchemaFilters,
        token: &str,
    ) -> Result<ConfigSchema, HTTPErr> {
        let search_query = build_search_query(filters.clone());
        let pagination = Pagination {
            limit: 1, offset: 0,
        };
        let query_params = build_query_params(
            search_query.as_deref(),
            None,
            &pagination,
        );

        let cfg_schemas = self.list_config_schemas(
            &query_params,
            token,
        ).await?;

        // check that there is only one config schema
        if cfg_schemas.data.len() > 1 {
            let ids = cfg_schemas.data.iter().map(|c| c.id.clone()).collect();
            return Err(HTTPErr::TooManyConfigSchemas(Box::new(TooManyConfigSchemas {
                expected_count: 1,
                found_config_schema_ids: ids,
                query_params,
                trace: trace!(),
            })));
        }

        match cfg_schemas.data.first() {
            Some(config_schema) => Ok(config_schema.clone()),
            None => Err(HTTPErr::ConfigSchemaNotFound(Box::new(ConfigSchemaNotFound {
                query_params,
                trace: trace!(),
            }))),
        }
    }

    async fn list_config_schemas(
        &self,
        query_params: &str,
        token: &str,
    ) -> Result<ConfigSchemaList, HTTPErr> {
        // build the request
        let url = format!("{}{}", self.config_schemas_url(), query_params);
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
        query_params: &str,
        token: &str,
    ) -> Result<ConfigSchemaList, HTTPErr> {
        self.as_ref().list_config_schemas(query_params, token).await
    }

    async fn find_one_config_schema(
        &self,
        filters: ConfigSchemaFilters,
        token: &str,
    ) -> Result<ConfigSchema, HTTPErr> {
        self.as_ref().find_one_config_schema(filters, token).await
    }
}

// ================================ SEARCH FILTERS ================================ //
#[derive(Debug, Clone)]
pub struct ConfigSchemaFilters {
    pub digests: Option<DigestFilter>,
    pub config_type_slugs: Option<ConfigTypeSlugFilter>,
}

#[derive(Debug, Clone)]
pub struct DigestFilter {
    pub not: bool,
    pub op: SearchOperator,
    pub val: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ConfigTypeSlugFilter {
    pub not: bool,
    pub op: SearchOperator,
    pub val: Vec<String>,
}

pub fn build_search_query(filters: ConfigSchemaFilters) -> Option<String> {
    let mut clauses: Vec<String> = Vec::new();
    if let Some(digests) = filters.digests {
        clauses.push(format_search_clause(
            ConfigSchemaSearch::CONFIG_SCHEMA_SEARCH_DIGEST,
            SearchOperator::Equals,
            digests.val,
            digests.not,
        ));
    }
    if let Some(config_type_slugs) = filters.config_type_slugs {
        clauses.push(format_search_clause(
            ConfigSchemaSearch::CONFIG_SCHEMA_SEARCH_CONFIG_TYPE_SLUG,
            SearchOperator::Equals,
            config_type_slugs.val,
            config_type_slugs.not,
        ));
    }
    format_search_group(clauses, LogicalOperator::And).map(|s| format!("search={}", s))
}
