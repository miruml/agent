// internal crates
use crate::env;
use crate::http::errors::{reqwest_err_to_http_client_err, HTTPErr};
use crate::trace;
// external crates
use async_once_cell::OnceCell;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::time::{sleep, timeout, Duration};
#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

// status codes
#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")] // Change case for all variants to snake_case
pub enum Code {
    Success,
    #[serde(rename = "invalid_jwt_auth")]
    InvalidJWTAuth,
    #[serde(rename = "invalid_rsa_auth")]
    InvalidRSAAuth,
    InvalidRequest,
    Unauthorized,
    ResourceNotFound,
    DeviceAlreadyAuthenticated,
    InternalServerError,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct APIResponse {
    pub data: Option<serde_json::Value>,
    pub message: String,
    pub code: Code,
    pub success: bool,
}

#[derive(Clone, Debug, Default)]
pub struct HTTPClient {
    // allow crate access since this struct is defined throughout the crate
    pub(crate) client: reqwest::Client,
    pub(crate) miru_base_url: String,
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
static CLIENT: OnceCell<reqwest::Client> = OnceCell::new();

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
        let mut base = "https://api.miruml.com".to_string();
        if env::ENV == "local" {
            base = "http://localhost:8080".to_string();
        } else if env::ENV == "dev" {
            base = "https://dev.api.miruml.com".to_string();
        }

        let client = CLIENT.get_or_init(init_client()).await;

        HTTPClient {
            client: client.clone(),
            miru_base_url: base + "/internal/devices/v1",
        }
    }

    pub fn artifact_route(&self, artifact_id: &str) -> String {
        format!("{}/artifacts/{}", self.miru_base_url, artifact_id)
    }

    pub fn device_route(&self, device_id: &str) -> String {
        format!("{}/devices/{}", self.miru_base_url, device_id)
    }

    pub fn job_route(&self, job_id: &str) -> String {
        format!("{}/jobs/{}", self.miru_base_url, job_id)
    }

    pub fn job_run_route(&self, job_run_id: &str) -> String {
        format!("{}/job-runs/{}", self.miru_base_url, job_run_id)
    }

    pub fn script_route(&self, script_id: &str) -> String {
        format!("{}/scripts/{}", self.miru_base_url, script_id)
    }

    pub fn script_run_route(&self, script_run_id: &str) -> String {
        format!("{}/script-runs/{}", self.miru_base_url, script_run_id)
    }

    /// Make a get request to the server using the provided url and optional token.
    pub async fn get(
        &self,
        url: &str,
        token: Option<&str>,
        time_limit: Duration,
    ) -> Result<reqwest::Response, HTTPErr> {
        // add token to headers if it exists
        let mut headers = HeaderMap::new();
        if let Some(token) = token {
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
        }

        // request server
        let response = timeout(time_limit, self.client.get(url).headers(headers).send())
            .await
            .map_err(|e| HTTPErr::TimeoutErr {
                msg: e.to_string(),
                timeout: time_limit,
                trace: trace!(),
            })?
            .map_err(|e| reqwest_err_to_http_client_err(e, trace!()))?;
        Ok(response)
    }

    pub async fn post<P>(
        &self,
        url: &str,
        data: &P,
        token: Option<&str>,
        time_limit: Duration,
    ) -> Result<reqwest::Response, HTTPErr>
    where
        P: Serialize,
    {
        // add token to headers if it exists
        let mut headers = HeaderMap::new();
        if let Some(token) = token {
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
        }

        // make the request
        let response = timeout(
            time_limit,
            self.client.post(url).headers(headers).json(data).send(),
        )
        .await
        .map_err(|e| HTTPErr::TimeoutErr {
            msg: e.to_string(),
            timeout: time_limit,
            trace: trace!(),
        })?
        .map_err(|e| reqwest_err_to_http_client_err(e, trace!()))?;

        Ok(response)
    }

    pub async fn get_miru_backend(
        &self,
        url: &str,
        token: Option<&str>,
        time_limit: Duration,
    ) -> Result<APIResponse, HTTPErr> {
        let resp = self.get(url, token, time_limit).await?;
        self.parse_reqwest_response(resp).await
    }

    // This function just makes a post request to the backend and returns the response
    // body and http status code without any additional processing. If the response is
    // not a success, the caller must handle the error.
    pub async fn post_miru_backend<P>(
        &self,
        url: &str,
        payload: &P,
        token: Option<&str>,
        time_limit: Duration,
    ) -> Result<APIResponse, HTTPErr>
    where
        P: Serialize,
    {
        let resp = self.post(url, payload, token, time_limit).await?;
        self.parse_reqwest_response(resp).await
    }

    async fn parse_reqwest_response(
        &self,
        response: reqwest::Response,
    ) -> Result<APIResponse, HTTPErr> {
        // get the api response
        let http_status = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| reqwest_err_to_http_client_err(e, trace!()))?;

        // parse as an APIResponse
        let resp =
            serde_json::from_str::<APIResponse>(&text).map_err(|e| HTTPErr::ParseJSONErr {
                source: e,
                trace: trace!(),
            })?;

        // check for errors
        match resp.code {
            Code::Success => Ok(resp),
            _ => {
                let msg = format!(
                    "GET Request failed with http status code '{}', miru error code '{}' and miru error message '{}'",
                    http_status, serde_json::to_string(&resp.code).unwrap_or_default(), resp.message
                );
                Err(HTTPErr::MiruResponseErr {
                    http_code: http_status,
                    code: resp.code,
                    msg,
                    trace: trace!(),
                })
            }
        }
    }

    pub async fn parse_api_response<R>(&self, api_resp: APIResponse) -> Result<R, HTTPErr>
    where
        R: DeserializeOwned,
    {
        // parse the response
        let data = match api_resp.data {
            Some(data) => data,
            None => {
                return Err(HTTPErr::ResponseDataMissingErr {
                    msg: "Response data missing".to_string(),
                    trace: trace!(),
                })
            }
        };
        serde_json::from_value::<R>(data).map_err(|e| HTTPErr::ParseJSONErr {
            source: e,
            trace: trace!(),
        })
    }
}

// Testing helper methods
impl HTTPClient {
    /// Set the Miru base URL for testing purposes. This method is only available
    /// in tests. Adjusting the Miru base URL is a simple way to test connectivity
    /// errors by inputting a URL that doesn't exist.
    ///
    /// ### Arguments
    ///
    /// * `url` - The URL to set as the Miru base URL.
    ///
    /// ### Returns
    ///
    /// Returns nothing.
    pub fn test_utils_set_miru_base_url(&mut self, url: &str) {
        self.miru_base_url = url.to_string();
    }
}
