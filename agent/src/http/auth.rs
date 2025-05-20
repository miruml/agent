// internal crates
use crate::http::client::HTTPClient;
use crate::http::errors::HTTPErr;
use openapi_client::models::{
    ActivateDeviceRequest, Device, IssueDeviceTokenRequest, TokenResponse,
};

// external crates
use async_trait::async_trait;

#[async_trait]
pub trait ClientAuthExt: Send + Sync {
    async fn activate_client(
        &self,
        client_id: &str,
        payload: &ActivateDeviceRequest,
        token: &str,
    ) -> Result<Device, HTTPErr>;

    async fn issue_client_token(
        &self,
        client_id: &str,
        payload: &IssueDeviceTokenRequest,
    ) -> Result<TokenResponse, HTTPErr>;
}

impl HTTPClient {
    fn clients_url(&self) -> String {
        format!("{}/clients", self.base_url)
    }

    fn client_url(&self, client_id: &str) -> String {
        format!("{}/{}", self.clients_url(), client_id)
    }
}

#[async_trait]
impl ClientAuthExt for HTTPClient {
    async fn activate_client(
        &self,
        client_id: &str,
        payload: &ActivateDeviceRequest,
        token: &str,
    ) -> Result<Device, HTTPErr> {
        // build the request
        let url = format!("{}/activate", self.client_url(client_id));
        let (request, context) = self.build_post_request(
            &url,
            self.marshal_json_payload(payload)?,
            self.default_timeout,
            Some(token),
        )?;

        // send the request (no caching)
        let http_resp = self.send(request, &context).await?;
        let text_resp = self.handle_response(http_resp, &context).await?;

        // parse the response
        self.parse_json_response_text::<Device>(text_resp, &context)
            .await
    }

    async fn issue_client_token(
        &self,
        client_id: &str,
        payload: &IssueDeviceTokenRequest,
    ) -> Result<TokenResponse, HTTPErr> {
        let url = format!("{}/issue_token", self.client_url(client_id));
        let (request, context) = self.build_post_request(
            &url,
            self.marshal_json_payload(payload)?,
            self.default_timeout,
            None,
        )?;

        // send the request (no caching)
        let http_resp = self.send(request, &context).await?;
        let text_resp = self.handle_response(http_resp, &context).await?;

        // parse the response
        self.parse_json_response_text::<TokenResponse>(text_resp, &context)
            .await
    }
}
