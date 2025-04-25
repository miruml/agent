// internal crates
use crate::http::client::HTTPClient;
use crate::http::errors::HTTPErr;
use openapi_client::models::{
    ActivateClientRequest, Client, IssueClientTokenRequest, IssueClientTokenResponse,
};

// external crates
use async_trait::async_trait;

#[async_trait]
pub trait ClientAuthExt: Send + Sync {
    async fn activate_client(
        &self,
        client_id: &str,
        payload: &ActivateClientRequest,
    ) -> Result<Client, HTTPErr>;

    async fn issue_client_token(
        &self,
        client_id: &str,
        payload: &IssueClientTokenRequest,
    ) -> Result<IssueClientTokenResponse, HTTPErr>;
}

impl HTTPClient {
    fn clients_url(&self) -> String {
        format!("{}/clients", self.base_url)
    }

    fn client_url(&self, client_id: &str) -> String {
        format!("{}/clients/{}", self.clients_url(), client_id)
    }
}

#[async_trait]
impl ClientAuthExt for HTTPClient {
    async fn activate_client(
        &self,
        client_id: &str,
        payload: &ActivateClientRequest,
    ) -> Result<Client, HTTPErr> {
        // build the request
        let url = format!("{}/activate", self.client_url(client_id));
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
        self.parse_json_response_text::<Client>(text_resp, &context)
            .await
    }

    async fn issue_client_token(
        &self,
        client_id: &str,
        payload: &IssueClientTokenRequest,
    ) -> Result<IssueClientTokenResponse, HTTPErr> {
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
        self.parse_json_response_text::<IssueClientTokenResponse>(text_resp, &context)
            .await
    }
}
