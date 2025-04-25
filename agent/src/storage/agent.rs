// external crates
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Agent {
    pub client_id: String,
}
