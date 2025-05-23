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
pub struct User {
    #[serde(rename = "object")]
    pub object: Object,
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "email")]
    pub email: String,
    #[serde(rename = "first_name", deserialize_with = "Option::deserialize")]
    pub first_name: Option<String>,
    #[serde(rename = "last_name", deserialize_with = "Option::deserialize")]
    pub last_name: Option<String>,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "updated_at")]
    pub updated_at: String,
    #[serde(rename = "workspace_id")]
    pub workspace_id: String,
    #[serde(rename = "workspace", deserialize_with = "Option::deserialize")]
    pub workspace: Option<Box<models::Workspace>>,
}

impl User {
    pub fn new(
        object: Object,
        id: String,
        email: String,
        first_name: Option<String>,
        last_name: Option<String>,
        created_at: String,
        updated_at: String,
        workspace_id: String,
        workspace: Option<models::Workspace>,
    ) -> User {
        User {
            object,
            id,
            email,
            first_name,
            last_name,
            created_at,
            updated_at,
            workspace_id,
            workspace: workspace.map(Box::new),
        }
    }
}
///
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Object {
    #[serde(rename = "user")]
    User,
}

impl Default for Object {
    fn default() -> Object {
        Self::User
    }
}
