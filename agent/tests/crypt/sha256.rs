#[cfg(test)]
mod tests {
    // internal crates
    use config_agent::crypt::sha256::{hash_bytes, hash_json, hash_str};
    use serde_json::json;
    // external crates
    #[allow(unused_imports)]
    use tracing::{debug, error, info, trace, warn};

    pub mod hash_json {
        use super::*;

        #[test]
        fn success() {
            let test_json = json!({
                "name": "test",
                "value": 123
            });
            let hash = hash_json(&test_json);
            // SHA-256 hash will be 64 characters long when hex encoded
            assert_eq!(hash.len(), 64);
            // Same input should produce same hash
            assert_eq!(hash, hash_json(&test_json));
        }
    }

    pub mod hash_str {
        use super::*;

        #[test]
        fn success() {
            let test_str = "hello world";
            let hash = hash_str(test_str);
            // SHA-256 hash will be 64 characters long when hex encoded
            assert_eq!(hash.len(), 64);
            // Same input should produce same hash
            assert_eq!(hash, hash_str(test_str));
            // Known hash for "hello world"
            assert_eq!(
                hash,
                "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
            );
        }
    }

    pub mod hash_bytes {
        use super::*;

        #[test]
        fn success() {
            let test_bytes = b"hello world";
            let hash = hash_bytes(test_bytes);
            // SHA-256 hash will be 64 characters long when hex encoded
            assert_eq!(hash.len(), 64);
            // Same input should produce same hash
            assert_eq!(hash, hash_bytes(test_bytes));
            // Known hash for "hello world"
            assert_eq!(
                hash,
                "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
            );
        }
    }
}
