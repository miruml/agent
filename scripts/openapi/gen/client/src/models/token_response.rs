/*
 * Miru API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.1.0
 * 
 * Generated by: https://openapi-generator.tech
 */

use crate::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct TokenResponse {
    /// The token
    #[serde(rename = "token")]
    pub token: String,
    /// The expiration
    #[serde(rename = "expires_at")]
    pub expires_at: String,
}

impl TokenResponse {
    pub fn new(token: String, expires_at: String) -> TokenResponse {
        TokenResponse {
            token,
            expires_at,
        }
    }
}

