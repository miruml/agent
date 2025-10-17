// internal crates
use crate::http::client::HTTPClient;
use crate::http::errors::HTTPErr;
use openapi_client::models::{
    ActivateDeviceRequest, Device, IssueDeviceTokenRequest, TokenResponse, UpdateDeviceFromAgentRequest,
};

#[allow(async_fn_in_trait)]
pub trait DevicesExt: Send + Sync {
    async fn activate_device(
        &self,
        device_id: &str,
        payload: &ActivateDeviceRequest,
        token: &str,
    ) -> Result<Device, HTTPErr>;

    async fn issue_device_token(
        &self,
        device_id: &str,
        payload: &IssueDeviceTokenRequest,
    ) -> Result<TokenResponse, HTTPErr>;

    async fn update_device(
        &self,
        device_id: &str,
        payload: &UpdateDeviceFromAgentRequest,
        token: &str,
    ) -> Result<Device, HTTPErr>;
}

impl HTTPClient {
    fn devices_url(&self) -> String {
        format!("{}/devices", self.base_url)
    }

    fn device_url(&self, device_id: &str) -> String {
        format!("{}/{}", self.devices_url(), device_id)
    }
}

impl DevicesExt for HTTPClient {
    async fn activate_device(
        &self,
        device_id: &str,
        payload: &ActivateDeviceRequest,
        token: &str,
    ) -> Result<Device, HTTPErr> {
        // build the request
        let url = format!("{}/activate", self.device_url(device_id));
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

    async fn issue_device_token(
        &self,
        device_id: &str,
        payload: &IssueDeviceTokenRequest,
    ) -> Result<TokenResponse, HTTPErr> {
        let url = format!("{}/issue_token", self.device_url(device_id));
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

    async fn update_device(
        &self,
        device_id: &str,
        payload: &UpdateDeviceFromAgentRequest,
        token: &str,
    ) -> Result<Device, HTTPErr> {
        let url = self.device_url(device_id);
        let (request, context) = self.build_patch_request(
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
}
