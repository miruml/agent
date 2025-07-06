// standard library
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

// internal crates
use crate::activity::ActivityTracker;
use crate::app::state::AppState;
use crate::auth::token_mngr::{TokenManager, TokenManagerExt};
use crate::deploy::fsm;
use crate::http::client::HTTPClient;
use crate::server::{
    errors::*,
    serve::{serve, ServerOptions},
    state::ServerState,
};
use crate::storage::{
    caches::CacheCapacities,
    layout::StorageLayout,
};
use crate::workers::{
        token_refresh::{
        run_token_refresh_worker,
        TokenRefreshWorkerOptions,
    },
    backend_sync::{
        run_backend_sync_worker,
        BackendSyncWorkerOptions,
    },
};
use crate::trace;

// external
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tracing::{error, info};

pub struct ServerComponents {
    pub state: Arc<ServerState>,
    pub state_handle: Pin<Box<dyn Future<Output = ()>>>,
    pub token_refresh_handle: JoinHandle<()>,
    pub server_handle: JoinHandle<Result<(), ServerErr>>,
}

#[derive(Debug, Clone, Copy)]
pub struct LifecycleOptions {
    pub is_persistent: bool,
    pub max_runtime: Duration,
    pub idle_timeout: Duration,
    pub idle_timeout_poll_interval: Duration,
    pub max_shutdown_delay: Duration,
}

impl Default for LifecycleOptions {
    fn default() -> Self {
        Self {
            is_persistent: true,
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

pub async fn run(
    options: AppOptions,
    shutdown_signal: impl Future<Output = ()> + Send + 'static,
) -> Result<(), ServerErr> {
    info!("Initializing miru agent...");

    // Create a single shutdown channel that all components will listen to
    let (shutdown_tx, _shutdown_rx): (tokio::sync::broadcast::Sender<()>, _) =
        tokio::sync::broadcast::channel(1);
    let mut shutdown_manager =
        ShutdownManager::new(
            shutdown_tx.clone(),
            options.lifecycle,
        );

    // initialize the app (and shutdown if failures occur)
    let app_state = match init(
        &options, shutdown_tx.clone(), &mut shutdown_manager,
    ).await {
        Ok(state) => state,
        Err(e) => {
            error!("Failed to start server: {}", e);
            shutdown_manager.shutdown().await?;
            return Err(e);
        }
    };

    // if the app is persistent, wait for ctrl-c to trigger a shutdown
    if options.lifecycle.is_persistent {
        tokio::select! {
            _ = shutdown_signal => {
                info!("Shutdown signal received, shutting down...");
            }
        }

    // if the app is not persistent, wait for ctrl-c, an idle timeout, or max runtime
    // reached to trigger a shutdown
    } else {
        tokio::select! {
            _ = shutdown_signal => {
                info!("Shutdown signal received, shutting down...");
            }
            _ = await_idle_timeout(
                app_state.activity_tracker.clone(),
                options.lifecycle.idle_timeout,
                options.lifecycle.idle_timeout_poll_interval,
            ) => {
                info!("Idle timeout ({:?}) reached", options.lifecycle.idle_timeout);
                info!("Shutting down...");
            }
            _ = await_max_runtime(options.lifecycle.max_runtime) => {
                info!("Max runtime ({:?}) reached, shutting down...", options.lifecycle.max_runtime);
            }
        }
    }

    // shutdown the server
    drop(shutdown_tx);
    shutdown_manager.shutdown().await
}

async fn await_idle_timeout(
    activity_tracker: Arc<ActivityTracker>,
    idle_timeout: Duration,
    poll_interval: Duration,
) -> Result<(), ServerErr> {
    loop {
        tokio::time::sleep(poll_interval).await;
        let last_activity = SystemTime::UNIX_EPOCH + Duration::from_secs(activity_tracker.last_touched());
        match SystemTime::now().duration_since(last_activity) {
            Ok(duration) if duration > idle_timeout => {
                info!("Server idle timeout reached, shutting down...");
                return Ok(());
            }
            Err(_) => {
                error!("Server idle timeout checker error, ignoring...");
            }
            _ => {}
        }
    }
}

async fn await_max_runtime(max_runtime: Duration) -> Result<(), ServerErr> {
    tokio::time::sleep(max_runtime).await;
    Ok(())
}

// =============================== INITIALIZATION ================================== //
async fn init(
    options: &AppOptions,
    shutdown_tx: broadcast::Sender<()>,
    shutdown_manager: &mut ShutdownManager,
) -> Result<Arc<AppState>, ServerErr> {

    let app_state = init_app_state(options, shutdown_manager).await?;

    init_token_refresh_worker(
        app_state.token_mngr.clone(),
        options.token_refresh_worker.clone(),
        shutdown_manager,
        shutdown_tx.subscribe(),
    ).await?;

    if options.enable_backend_sync_worker {
        init_backend_sync_worker(
            options.backend_sync_worker.clone(),
            app_state.clone(),
            shutdown_manager,
            shutdown_tx.subscribe(),
        ).await?;
    }

    if options.enable_socket_server {
        init_socket_server(
            options,
            app_state.clone(),
            shutdown_manager,
            shutdown_tx.subscribe(),
        ).await?;
    }


    Ok(app_state)
}

async fn init_app_state(
    options: &AppOptions,
    shutdown_manager: &mut ShutdownManager,
) -> Result<Arc<AppState>, ServerErr> {

    let (app_state, app_state_handle) = AppState::init(
        &options.storage.layout,
        options.storage.cache_capacities,
        Arc::new(HTTPClient::new(&options.backend_base_url).await),
        options.fsm_settings,
    )
    .await?;
    let app_state = Arc::new(app_state);
    shutdown_manager.with_app_state(
        app_state.clone(),
        Box::pin(app_state_handle),
    )?;

    Ok(app_state)
}

async fn init_token_refresh_worker(
    token_mngr: Arc<TokenManager>,
    options: TokenRefreshWorkerOptions,
    shutdown_manager: &mut ShutdownManager,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Result<(), ServerErr> {
    info!("Initializing token refresh worker...");

    // refresh the token before starting the refresh worker if it is expired
    if let Err(e) = refresh_if_expired(&token_mngr).await {
        error!("Failed to refresh expired token: {}", e);
    }

    // start the refresh worker
    let token_refresh_handle = tokio::spawn(async move {
        run_token_refresh_worker(
            &options,
            token_mngr.as_ref(),
            |wait| tokio::time::sleep(wait),
            Box::pin(async move {
                let _ = shutdown_rx.recv().await;
            }),
        )
        .await;
    });
    shutdown_manager.with_token_refresh_worker_handle(token_refresh_handle)?;
    Ok(())
}

async fn refresh_if_expired(token_mngr: &TokenManager) -> Result<(), ServerErr> {
    let token = token_mngr.get_token().await.map_err(|e| {
        ServerErr::AuthErr(Box::new(ServerAuthErr {
            source: e,
            trace: trace!(),
        }))
    })?;
    if token.is_expired() {
        token_mngr.refresh_token().await.map_err(|e| {
            ServerErr::AuthErr(Box::new(ServerAuthErr {
                source: e,
                trace: trace!(),
            }))
        })?;
    }
    Ok(())
}

async fn init_backend_sync_worker(
    options: BackendSyncWorkerOptions,
    app_state: Arc<AppState>,
    shutdown_manager: &mut ShutdownManager,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Result<(), ServerErr> {
    info!("Initializing backend sync worker...");

    let device_id = app_state.device_id.clone();
    let token_mngr = app_state.token_mngr.clone();
    let syncer = app_state.syncer.clone();
    
    let backend_sync_handle = tokio::spawn(async move {
        run_backend_sync_worker(
            device_id.as_ref(),
            &options,
            token_mngr.as_ref(),
            syncer.as_ref(),
            Box::pin(async move {
                let _ = shutdown_rx.recv().await;
            }),
        )
        .await;
    });
    shutdown_manager.with_backend_sync_worker_handle(backend_sync_handle)?;
    Ok(())
}

async fn init_socket_server(
    options: &AppOptions,
    app_state: Arc<AppState>,
    shutdown_manager: &mut ShutdownManager,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Result<(), ServerErr> {
    info!("Initializing socket server...");

    // run the axum server with graceful shutdown
    let server_state = ServerState::new(
        app_state.device_id.clone(),
        app_state.http_client.clone(),
        app_state.syncer.clone(),
        app_state.caches.clone(),
        app_state.token_mngr.clone(),
        app_state.activity_tracker.clone(),
    );
    let server_handle = serve(
        &options.server,
        Arc::new(server_state),
        async move {
            let _ = shutdown_rx.recv().await;
        },
    )
    .await?;
    shutdown_manager.with_socket_server_handle(server_handle)?;

    Ok(())
}

// ================================= SHUTDOWN ===================================== //
struct AppStateShutdownParams {
    state: Arc<AppState>,
    state_handle: Pin<Box<dyn Future<Output = ()> + Send>>,
}

struct ShutdownManager {
    // shutdown transmitter
    shutdown_tx: broadcast::Sender<()>,
    lifecycle_options: LifecycleOptions,

    // server components requiring shutdown
    app_state: Option<AppStateShutdownParams>,
    token_refresh_worker_handle: Option<JoinHandle<()>>,
    backend_sync_worker_handle: Option<JoinHandle<()>>,
    socket_server_handle: Option<JoinHandle<Result<(), ServerErr>>>,
}

impl ShutdownManager {
    pub fn new(shutdown_tx: broadcast::Sender<()>, lifecycle_options: LifecycleOptions) -> Self {
        Self {
            shutdown_tx,
            lifecycle_options,
            app_state: None,
            token_refresh_worker_handle: None,
            backend_sync_worker_handle: None,
            socket_server_handle: None,
        }
    }

    pub fn with_app_state(
        &mut self,
        state: Arc<AppState>,
        state_handle: Pin<Box<dyn Future<Output = ()> + Send>>,
    ) -> Result<(), ServerErr> {
        if self.app_state.is_some() {
            return Err(ServerErr::ShutdownMngrDuplicateArgErr(Box::new(
                ShutdownMngrDuplicateArgErr {
                    arg_name: "app_state".to_string(),
                    trace: trace!(),
                },
            )));
        }
        self.app_state = Some(AppStateShutdownParams { state, state_handle });
        Ok(())
    }

    pub fn with_token_refresh_worker_handle(
        &mut self,
        token_refresh_handle: JoinHandle<()>,
    ) -> Result<(), ServerErr> {
        if self.token_refresh_worker_handle.is_some() {
            return Err(ServerErr::ShutdownMngrDuplicateArgErr(Box::new(
                ShutdownMngrDuplicateArgErr {
                    arg_name: "token_refresh_handle".to_string(),
                    trace: trace!(),
                },
            )));
        }
        self.token_refresh_worker_handle = Some(token_refresh_handle);
        Ok(())
    }

    pub fn with_backend_sync_worker_handle(
        &mut self,
        backend_sync_handle: JoinHandle<()>,
    ) -> Result<(), ServerErr> {
        if self.backend_sync_worker_handle.is_some() {
            return Err(ServerErr::ShutdownMngrDuplicateArgErr(Box::new(
                ShutdownMngrDuplicateArgErr {
                    arg_name: "backend_sync_handle".to_string(),
                    trace: trace!(),
                },
            )));
        }
        self.backend_sync_worker_handle = Some(backend_sync_handle);
        Ok(())
    }

    pub fn with_socket_server_handle(
        &mut self,
        socket_server_handle: JoinHandle<Result<(), ServerErr>>,
    ) -> Result<(), ServerErr> {
        if self.socket_server_handle.is_some() {
            return Err(ServerErr::ShutdownMngrDuplicateArgErr(Box::new(
                ShutdownMngrDuplicateArgErr {
                    arg_name: "server_handle".to_string(),
                    trace: trace!(),
                },
            )));
        }
        self.socket_server_handle = Some(socket_server_handle);
        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<(), ServerErr> {
        // send the shutdown signal to all components
        let _ = self.shutdown_tx.send(());

        match tokio::time::timeout(
            self.lifecycle_options.max_shutdown_delay,
            self.shutdown_impl(),
        ).await {
            Ok(result) => result,
            Err(_) => {
                error!(
                    "Shutdown timed out after {:?}, forcing shutdown...",
                    self.lifecycle_options.max_shutdown_delay
                );
                std::process::exit(1);
            }
        }
    }

    async fn shutdown_impl(&mut self) -> Result<(), ServerErr> {
        // the shutdown order is important here. The refresh and server threads rely on
        // the state so the state must be shutdown last.
        info!("Shutting down miru agent...");

        // 1. refresh
        if let Some(token_refresh_worker_handle) = self.token_refresh_worker_handle.take() {
            token_refresh_worker_handle.await.map_err(|e| {
                ServerErr::JoinHandleErr(Box::new(JoinHandleErr {
                    source: Box::new(e),
                    trace: trace!(),
                }))
            })?;
        } else {
            info!("Token refresh worker handle not found, skipping token refresh worker shutdown...");
        }

        // 2. backend sync
        if let Some(backend_sync_worker_handle) = self.backend_sync_worker_handle.take() {
            backend_sync_worker_handle.await.map_err(|e| {
                ServerErr::JoinHandleErr(Box::new(JoinHandleErr {
                    source: Box::new(e),
                    trace: trace!(),
                }))
            })?;
        } else {
            info!("Backend sync worker handle not found, skipping backend sync worker shutdown...");
        }

        // 3. server
        if let Some(socket_server_handle) = self.socket_server_handle.take() {
            socket_server_handle.await.map_err(|e| {
                ServerErr::JoinHandleErr(Box::new(JoinHandleErr {
                    source: Box::new(e),
                    trace: trace!(),
                }))
            })??;
        } else {
            info!("Socket server handle not found, skipping socket server shutdown...");
        }

        // 4. app state
        if let Some(app_state) = self.app_state.take() {
            app_state.state.shutdown().await?;
            app_state.state_handle.await;
        } else {
            info!("App state not found, skipping app state shutdown...");
        }

        info!("Program shutdown complete");
        Ok(())
    }
}