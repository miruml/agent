// standard library
use std::sync::Arc;

// internal crates
use crate::crypt::sha256;
use crate::http::client::HTTPClient;
use crate::http::errors::HTTPErr;
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

    async fn list_config_schemas<'a, D, S>(
        &self,
        digests: D,
        config_type_slugs: S,
        token: &str,
    ) -> Result<ConfigSchemaList, HTTPErr>
    where
        D: IntoIterator<Item = &'a str>,
        S: IntoIterator<Item = &'a str>,
    ;

    async fn find_one_config_schema<'a, D, S>(
        &self,
        digests: D,
        config_type_slugs: S,
        token: &str,
    ) -> Result<ConfigSchema, HTTPErr>
    where
        D: IntoIterator<Item = &'a str> + Clone,
        S: IntoIterator<Item = &'a str> + Clone,
    ;
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

    async fn find_one_config_schema<'a, D, S>(
        &self,
        digests: D,
        config_type_slugs: S,
        token: &str,
    ) -> Result<ConfigSchema, HTTPErr>
    where
        D: IntoIterator<Item = &'a str> + Clone,
        S: IntoIterator<Item = &'a str> + Clone,
    {
        let cfg_schemas = self.list_config_schemas(
            digests.clone(), config_type_slugs.clone(), token,
        ).await?;

        // check that there is only one config schema
        if cfg_schemas.data.len() > 1 {
            let ids = cfg_schemas.data.iter().map(|c| c.id.clone()).collect();
            return Err(HTTPErr::TooManyConfigSchemas(TooManyConfigSchemas {
                expected_count: 1,
                found_config_schema_ids: ids,
                config_type_slugs: config_type_slugs.into_iter().map(|s| s.to_string()).collect(),
                config_schema_digests: digests.into_iter().map(|d| d.to_string()).collect(),
                trace: trace!(),
            }));
        }

        match cfg_schemas.data.first() {
            Some(config_schema) => Ok(config_schema.clone()),
            None => Err(HTTPErr::ConfigSchemaNotFound(ConfigSchemaNotFound {
                config_type_slugs: config_type_slugs.into_iter().map(|s| s.to_string()).collect(),
                config_schema_digests: digests.into_iter().map(|d| d.to_string()).collect(),
                trace: trace!(),
            })),
        }
    }

    async fn list_config_schemas<'a, D, S>(
        &self,
        digests: D,
        config_type_slugs: S,
        token: &str,
    ) -> Result<ConfigSchemaList, HTTPErr>
    where
        D: IntoIterator<Item = &'a str>,
        S: IntoIterator<Item = &'a str>,
    {
        // build the search query
        let mut clauses: Vec<String> = Vec::new();
        let mut digests_iter = digests.into_iter().peekable();
        if digests_iter.peek().is_some() {
            clauses.push(format_search_clause(
                ConfigSchemaSearch::CONFIG_SCHEMA_SEARCH_DIGEST,
                SearchOperator::Equals,
                digests_iter,
            ));
        }
        let mut config_type_slugs_iter = config_type_slugs.into_iter().peekable();
        if config_type_slugs_iter.peek().is_some() {
            clauses.push(format_search_clause(
                ConfigSchemaSearch::CONFIG_SCHEMA_SEARCH_CONFIG_TYPE_SLUG,
                SearchOperator::Equals,
                config_type_slugs_iter,
            ));
        }

        let query = if clauses.is_empty() {
            "".to_string()
        } else {
            let search_query = format_search_group(clauses, LogicalOperator::And);
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

    async fn list_config_schemas<'a, D, S>(
        &self,
        digests: D,
        config_type_slugs: S,
        token: &str,
    ) -> Result<ConfigSchemaList, HTTPErr>
    where
        D: IntoIterator<Item = &'a str>,
        S: IntoIterator<Item = &'a str>,
    {
        self.as_ref().list_config_schemas(digests, config_type_slugs, token).await
    }

    async fn find_one_config_schema<'a, D, S>(
        &self,
        digests: D,
        config_type_slugs: S,
        token: &str,
    ) -> Result<ConfigSchema, HTTPErr>
    where
        D: IntoIterator<Item = &'a str> + Clone,
        S: IntoIterator<Item = &'a str> + Clone,
    {
        self.as_ref().find_one_config_schema(digests, config_type_slugs, token).await
    }
}