// external crates
use serde_json::Value;
use sha2::{Digest, Sha256};

pub fn hash_json(json: &Value) -> String {
    hash_bytes(json.to_string().as_bytes())
}

pub fn hash_str(s: &str) -> String {
    hash_bytes(s.as_bytes())
}

pub fn hash_bytes(bytes: &[u8]) -> String {
    let hash = Sha256::digest(bytes);
    format!("{:x}", hash)
}
