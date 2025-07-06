// standard library
use std::time::Duration;

// internal crates
use crate::deploy::fsm;
use crate::server::{
    serve::ServerOptions,
};
use crate::storage::{
    caches::CacheCapacities,
    layout::StorageLayout,
};
use crate::workers::{
    token_refresh::TokenRefreshWorkerOptions,
    backend_sync::BackendSyncWorkerOptions,
};


#[derive(Debug, Clone, Copy)]
pub struct LifecycleOptions {
    pub is_socket_activated: bool,
    pub max_runtime: Duration,
    pub idle_timeout: Duration,
    pub idle_timeout_poll_interval: Duration,
    pub max_shutdown_delay: Duration,
}

impl Default for LifecycleOptions {
    fn default() -> Self {
        Self {
            is_socket_activated: true,
            max_runtime: Duration::from_secs(60 * 15), // 15 minutes
            idle_timeout: Duration::from_secs(60),
            idle_timeout_poll_interval: Duration::from_secs(5),
            max_shutdown_delay: Duration::from_secs(15),
        }
    }
}

#[derive(Debug, Default)]
pub struct StorageOptions {
    pub layout: StorageLayout,
    pub cache_capacities: CacheCapacities,
}

#[derive(Debug)]
pub struct AppOptions {
    pub lifecycle: LifecycleOptions,

    pub storage: StorageOptions,
    pub token_refresh_worker: TokenRefreshWorkerOptions,
    pub fsm_settings: fsm::Settings,

    pub backend_base_url: String,

    pub enable_socket_server: bool,
    pub server: ServerOptions,

    pub enable_backend_sync_worker: bool,
    pub backend_sync_worker: BackendSyncWorkerOptions,
}

impl Default for AppOptions {
    fn default() -> Self {
        Self {
            lifecycle: LifecycleOptions::default(),

            storage: StorageOptions::default(),
            token_refresh_worker: TokenRefreshWorkerOptions::default(),
            fsm_settings: fsm::Settings::default(),

            backend_base_url: "https://configs.api.miruml.com/agent/v1".to_string(),

            enable_socket_server: true,
            server: ServerOptions::default(),

            enable_backend_sync_worker: true,
            backend_sync_worker: BackendSyncWorkerOptions::default(),
        }
    }
}