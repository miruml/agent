// std
use std::sync::Arc;

// internal crates
use crate::env;
use crate::errors::MiruError;
use crate::http_client::errors::{reqwest_err_to_http_client_err, HTTPErr};
use crate::trace;
use openapi_client::models::ErrorResponse;

// external crates
use moka::future::Cache;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::time::{sleep, timeout, Duration};
use tokio::sync::OnceCell;
use uuid::Uuid;

// type aliases
type RequestKey = String;
type Response = String;
type RequestID = Uuid;
type IsCacheHit = bool;

// status codes
#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")] // Change case for all variants to snake_case
pub enum Code {
    Success,
    InternalServerError,
    #[serde(other)]
    Unknown,
}

#[derive(Debug)]
pub struct HTTPClient {
    // allow crate access since this struct is defined throughout the crate
    pub(crate) client: reqwest::Client,
    pub(crate) base_url: String,
    pub(crate) timeout: Duration,
    cache: Cache<RequestKey, (Response, RequestID)>,
}

// Use Lazy to implement the Singleton(ish) Pattern for the reqwest client (see the
// README for more information). Per the documentation, we do not need to wrap the
// client in Rc or Arc to reuse it in a thread safe manner since reqwest already handles
// this under the hood [1]. Thus, our job is easy: just initialize the client and clone
// when wanting to reuse it [2]. One last note, avoid the reqwest::Client::new() method
// since it panics on a failure. Instead, use the reqwest::Client::builder() method so
// we can handle the failure gracefully [3].

// Sources:
// 1. https://docs.rs/reqwest/latest/reqwest/struct.Client.html
// 2. https://users.rust-lang.org/t/reqwest-http-client-fails-when-too-much-concurrency/55644
// 3. https://docs.rs/reqwest/latest/reqwest/struct.Client.html#method.builder
static CLIENT: OnceCell<reqwest::Client> = OnceCell::const_new();

async fn init_client() -> reqwest::Client {
    loop {
        let client = reqwest::Client::builder().build();
        if let Ok(client) = client {
            return client;
        }
        // wait 60 seconds before trying again
        sleep(Duration::from_secs(60)).await;
    }
}

impl HTTPClient {
    pub async fn new() -> Self {
        // default to production
        let mut base = "https://configs.api.miruml.com".to_string();
        if env::ENV == "local" {
            base = "http://localhost:8080".to_string();
        } else if env::ENV == "dev" {
            base = "https://dev.api.miruml.com".to_string();
        }

        let client = CLIENT.get_or_init(init_client).await;

        HTTPClient {
            client: client.clone(),
            base_url: base + "/internal/devices/v1",
            timeout: Duration::from_secs(10),
            cache: Cache::builder()
                .time_to_live(Duration::from_secs(30))
                .build(),
        }
    }

    fn add_token_to_headers(
        &self,
        headers: &mut HeaderMap,
        token: &str,
    ) -> Result<(), HTTPErr> {
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).map_err(|e| {
                HTTPErr::InvalidHeaderValueErr {
                    msg: e.to_string(),
                    source: e,
                    trace: trace!(),
                }
            })?,
        );
        Ok(())
    }

    fn build_request(
        &self,
        method: reqwest::Method,
        url: &str,
        body: Option<String>,
        token: Option<&str>,
    ) -> Result<reqwest::RequestBuilder, HTTPErr> {
        // request type (GET, POST, etc.)
        let mut request = self.client.request(method, url);
        
        // headers
        let mut headers = HeaderMap::new();
        if let Some(token) = token {
            self.add_token_to_headers(&mut headers, token)?;
        }
        request = request.headers(headers);

        // body
        if let Some(body) = body {
            request = request.body(body);
        }
        Ok(request)
    }

    pub fn build_get_request(
        &self,
        url: &str,
        token: Option<&str>,
    ) -> Result<reqwest::RequestBuilder, HTTPErr> {
        self.build_request(reqwest::Method::GET, url, None, token)
    }

    pub fn build_post_request(
        &self,
        url: &str,
        body: String,
        token: Option<&str>,
    ) -> Result<reqwest::RequestBuilder, HTTPErr> {
        self.build_request(reqwest::Method::POST, url, Some(body), token)
    }

    pub async fn send(
        &self,
        request: reqwest::RequestBuilder,
        time_limit: Duration,
    ) -> Result<reqwest::Response, HTTPErr> {
        // request server
        let response = timeout(time_limit, request.send())
            .await
            .map_err(|e| HTTPErr::TimeoutErr {
                msg: e.to_string(),
                timeout: time_limit,
                trace: trace!(),
            })?
            .map_err(|e| reqwest_err_to_http_client_err(e, trace!()))?;
        Ok(response)
    }

    pub async fn send_cached(
        &self,
        key: RequestKey,
        request: reqwest::RequestBuilder,
        time_limit: Duration,
    ) -> Result<(String, IsCacheHit), HTTPErr> {
        let id = Uuid::new_v4();
        let result = self.cache
            .try_get_with(
                key,
                async move {
                    let response = self.send(request, time_limit).await?;
                    Ok((self.handle_response(response).await?, id))
                }
            )
            .await
            .map_err(|e: Arc<HTTPErr>| HTTPErr::CacheErr {
                is_network_connection_error: e.is_network_connection_error(),
                msg: e.to_string(),
                trace: trace!(),
            })?;
        let is_cache_hit = result.1 != id;
        Ok((result.0, is_cache_hit))
    }

    pub fn marshal_json_request<T>(
        &self,
        request: &T,
    ) -> Result<String, HTTPErr> where T: Serialize {
        serde_json::to_string(request).map_err(|e| HTTPErr::ParseJSONErr {
            source: e,
            trace: trace!(),
        })
    }

    pub async fn handle_response(
        &self,
        response: reqwest::Response,
    ) -> Result<String, HTTPErr> {
        let status = response.status();

        // check for an error response
        if !status.is_success() {
            let url = response.url().to_string();
            let error_response = match response.text().await {
                Ok(text) => self.parse_json_response_text::<ErrorResponse>(text).await.ok(),
                Err(_) => None,
            };
            return Err(HTTPErr::ResponseFailed {
                status,
                url,
                error: error_response,
                trace: trace!(),
            });
        }

        let text = response
            .text()
            .await
            .map_err(|e| reqwest_err_to_http_client_err(e, trace!()))?;
        Ok(text)
    }

    pub(crate) async fn parse_json_response_text<T>(
        &self,
        text: String,
    ) -> Result<T, HTTPErr> where T: DeserializeOwned {
        serde_json::from_str::<T>(&text).map_err(|e| HTTPErr::ParseJSONErr {
            source: e,
            trace: trace!(),
        })
    }

    #[doc(hidden)]
    pub fn new_with(
        base_url: &str,
        timeout: Duration,
        cache: Cache<String, (String, Uuid)>,
    ) -> Self {
        HTTPClient {
            client: reqwest::Client::new(),
            base_url: base_url.to_string(),
            timeout,
            cache,
        }
    }
}

// Testing helper methods
#[cfg(test)]
impl HTTPClient {
    pub fn test_utils_set_base_url(&mut self, url: &str) {
        self.base_url = url.to_string();
    }

    pub fn test_utils_get_cache(&mut self) -> &mut Cache<String, (String, Uuid)> {
        &mut self.cache
    }
}
