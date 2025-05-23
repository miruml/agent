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

use serde_with::serde_as;

#[serde_as]
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct HashSchemaSerializedRequest {
    #[serde(rename = "format")]
    pub format: models::HashSerializedConfigSchemaFormat,
    #[serde_as(as = "serde_with::base64::Base64")]
    #[serde(rename = "schema")]
    pub schema: Vec<u8>,
}

impl HashSchemaSerializedRequest {
    pub fn new(format: models::HashSerializedConfigSchemaFormat, schema: Vec<u8>) -> HashSchemaSerializedRequest {
        HashSchemaSerializedRequest {
            format,
            schema,
        }
    }
}

