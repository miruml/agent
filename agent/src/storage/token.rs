// external crates
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Token {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

impl Token {
    pub fn is_expired(&self) -> bool {
        self.expires_at < Utc::now()
    }
}
