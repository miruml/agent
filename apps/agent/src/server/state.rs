// standard library
use std::sync::Arc;

// internal crates
use crate::auth::token_mngr::TokenManager;
use crate::http::client::HTTPClient;
use crate::storage::{
    caches::Caches,
};
use crate::sync::syncer::Syncer;
use crate::activity::ActivityTracker;


#[derive(Clone, Debug)]
pub struct ServerState {
    pub device_id: String,
    pub http_client: Arc<HTTPClient>,
    pub syncer: Arc<Syncer>,
    pub caches: Arc<Caches>,
    pub token_mngr: Arc<TokenManager>,
    pub activity_tracker: Arc<ActivityTracker>,
}

impl ServerState {
    pub fn new(
        device_id: String,
        http_client: Arc<HTTPClient>,
        syncer: Arc<Syncer>,
        caches: Arc<Caches>,
        token_mngr: Arc<TokenManager>,
        activity_tracker: Arc<ActivityTracker>,
    ) -> Self {
        ServerState {
            device_id,
            http_client,
            syncer,
            caches,
            token_mngr,
            activity_tracker,
        }
    }
}
