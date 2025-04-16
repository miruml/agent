// external crates
use sha2::{Sha256, Digest};
use serde_json::Value;

pub fn hash_json(json: &Value) -> String {
    hash_bytes(json.to_string().as_bytes())
}

pub fn hash_bytes(bytes: &[u8]) -> String {
    let hash = Sha256::digest(bytes);
    hex::encode(hash)
}