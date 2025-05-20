// standard library
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

// internal crates
use crate::auth::token_mngr::run_refresh_loop;
use crate::filesys::file::File;
use crate::http::client::HTTPClient;
use crate::server::errors::{JoinHandleErr, ServerErr, ShutdownMngrDuplicateArgErr};
use crate::server::serve::serve;
use crate::server::state::ServerState;
use crate::storage::layout::StorageLayout;
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

pub struct RunServerOptions {
    // host os
    pub layout: StorageLayout,
    pub backend_base_url: String,
    pub socket_file: File,

    // caches
    pub config_schema_digest_cache_max_size: usize,
    pub config_instance_cache_max_size: usize,

    // timing
    pub token_refresh_expiration_threshold: Duration,
    pub token_refresh_cooldown: Duration,
    pub max_runtime: Duration,
    pub idle_timeout: Duration,
    pub idle_timeout_poll_interval: Duration,
    pub max_shutdown_delay: Duration,
}

impl Default for RunServerOptions {
    fn default() -> Self {
        Self {
            // host os
            socket_file: File::new("/run/miru/miru.sock"),
            backend_base_url: "https://configs.api.miruml.com/agent/v1".to_string(),
            layout: StorageLayout::default(),

            // caches
            config_schema_digest_cache_max_size: 1000,
            config_instance_cache_max_size: 1000,

            // timing
            token_refresh_expiration_threshold: Duration::from_secs(15 * 60), // 15 minutes
            token_refresh_cooldown: Duration::from_secs(30),
            max_runtime: Duration::from_secs(60 * 15), // 15 minutes
            idle_timeout: Duration::from_secs(60),
            idle_timeout_poll_interval: Duration::from_secs(5),
            max_shutdown_delay: Duration::from_secs(15),
        }
    }
}

pub async fn run(
    options: RunServerOptions,
    shutdown_signal: impl Future<Output = ()> + Send + 'static,
) -> Result<(), ServerErr> {
    info!("Starting miru agent...");

    // Create a single shutdown channel that all components will listen to
    let (shutdown_tx, _shutdown_rx): (tokio::sync::broadcast::Sender<()>, _) =
        tokio::sync::broadcast::channel(1);
    let mut shutdown_manager =
        ShutdownManager::new(shutdown_tx.clone(), options.max_shutdown_delay);
    let shutdown_rx_refresh = shutdown_tx.subscribe();
    let shutdown_rx2_server = shutdown_tx.subscribe();

    // start the server (and shutdown if failures occur)
    let state = match start_server(
        &options,
        &mut shutdown_manager,
        shutdown_rx_refresh,
        shutdown_rx2_server,
    )
    .await
    {
        Ok(state) => state,
        Err(e) => {
            error!("Failed to start server: {}", e);
            shutdown_manager.shutdown().await?;
            return Err(e);
        }
    };

    // wait for ctrl-c, an idle timeout, or max runtime reached to trigger a shutdown
    tokio::select! {
        _ = shutdown_signal => {
            info!("Shutdown signal received, shutting down...");
        }
        _ = await_idle_timeout(
            state.last_activity.clone(),
            options.idle_timeout,
            options.idle_timeout_poll_interval,
        ) => {
            info!("Idle timeout ({:?}) reached", options.idle_timeout);
            info!("Pruning filesystem cache...");
            if let Err(e) = state.config_schema_digest_cache.prune(options.config_schema_digest_cache_max_size).await {
                error!("Failed to prune config schema digest cache: {}", e);
            }
            if let Err(e) = state.config_instance_cache.prune(options.config_instance_cache_max_size).await {
                error!("Failed to prune config instance cache: {}", e);
            }
            info!("Shutting down...");
        }
        _ = await_max_runtime(options.max_runtime) => {
            info!("Max runtime ({:?}) reached, shutting down...", options.max_runtime);
        }
    }

    // shutdown the server
    drop(shutdown_tx);
    shutdown_manager.shutdown().await
}

async fn start_server(
    options: &RunServerOptions,
    shutdown_manager: &mut ShutdownManager,
    mut shutdown_rx_refresh: broadcast::Receiver<()>,
    mut shutdown_rx2_server: broadcast::Receiver<()>,
) -> Result<Arc<ServerState>, ServerErr> {
    // initialize the server state
    let (state, state_handle) = ServerState::new(
        options.layout.clone(),
        Arc::new(HTTPClient::new(&options.backend_base_url).await),
    )
    .await?;
    let state = Arc::new(state);
    shutdown_manager.with_state(state.clone(), Box::pin(state_handle))?;

    // initialize the token refresh loop
    let token_mngr_for_spawn = state.token_mngr.clone();
    let token_refresh_expiration_threshold = options.token_refresh_expiration_threshold;
    let token_refresh_cooldown = options.token_refresh_cooldown;
    let token_refresh_handle = tokio::spawn(async move {
        run_refresh_loop(
            token_mngr_for_spawn,
            token_refresh_expiration_threshold,
            token_refresh_cooldown,
            async move {
                let _ = shutdown_rx_refresh.recv().await;
            },
        )
        .await;
    });
    shutdown_manager.with_token_refresh_handle(token_refresh_handle)?;

    // run the axum server with graceful shutdown
    let server_handle = serve(&options.socket_file, state.clone(), async move {
        let _ = shutdown_rx2_server.recv().await;
    })
    .await?;
    shutdown_manager.with_server_handle(server_handle)?;

    Ok(state)
}

async fn await_idle_timeout(
    shared_last_activity: Arc<AtomicU64>,
    idle_timeout: Duration,
    poll_interval: Duration,
) -> Result<(), ServerErr> {
    loop {
        tokio::time::sleep(poll_interval).await;
        let last_activity = SystemTime::UNIX_EPOCH
            + Duration::from_secs(shared_last_activity.load(Ordering::Relaxed));
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

// ============================== SHUTDOWN MANAGER ================================ //
struct StateShutdownParams {
    state: Arc<ServerState>,
    state_handle: Pin<Box<dyn Future<Output = ()> + Send>>,
}

struct ShutdownManager {
    // shutdown transmitter
    shutdown_tx: broadcast::Sender<()>,

    // server components requiring shutdown
    state_params: Option<StateShutdownParams>,
    token_refresh_handle: Option<JoinHandle<()>>,
    server_handle: Option<JoinHandle<Result<(), ServerErr>>>,

    // shutdown options
    max_shutdown_delay: Duration,
}

impl ShutdownManager {
    pub fn new(shutdown_tx: broadcast::Sender<()>, max_shutdown_delay: Duration) -> Self {
        Self {
            shutdown_tx,
            state_params: None,
            token_refresh_handle: None,
            server_handle: None,
            max_shutdown_delay,
        }
    }

    pub fn with_state(
        &mut self,
        state: Arc<ServerState>,
        state_handle: Pin<Box<dyn Future<Output = ()> + Send>>,
    ) -> Result<(), ServerErr> {
        if self.state_params.is_some() {
            return Err(ServerErr::ShutdownMngrDuplicateArgErr(
                ShutdownMngrDuplicateArgErr {
                    arg_name: Box::new("state".to_string()),
                    trace: trace!(),
                },
            ));
        }
        self.state_params = Some(StateShutdownParams {
            state,
            state_handle,
        });
        Ok(())
    }

    pub fn with_token_refresh_handle(
        &mut self,
        token_refresh_handle: JoinHandle<()>,
    ) -> Result<(), ServerErr> {
        if self.token_refresh_handle.is_some() {
            return Err(ServerErr::ShutdownMngrDuplicateArgErr(
                ShutdownMngrDuplicateArgErr {
                    arg_name: Box::new("token_refresh_handle".to_string()),
                    trace: trace!(),
                },
            ));
        }
        self.token_refresh_handle = Some(token_refresh_handle);
        Ok(())
    }

    pub fn with_server_handle(
        &mut self,
        server_handle: JoinHandle<Result<(), ServerErr>>,
    ) -> Result<(), ServerErr> {
        if self.server_handle.is_some() {
            return Err(ServerErr::ShutdownMngrDuplicateArgErr(
                ShutdownMngrDuplicateArgErr {
                    arg_name: Box::new("server_handle".to_string()),
                    trace: trace!(),
                },
            ));
        }
        self.server_handle = Some(server_handle);
        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<(), ServerErr> {
        // send the shutdown signal to all components
        let _ = self.shutdown_tx.send(());

        match tokio::time::timeout(self.max_shutdown_delay, self.shutdown_impl()).await {
            Ok(result) => result,
            Err(_) => {
                error!(
                    "Shutdown timed out after {:?}, forcing shutdown...",
                    self.max_shutdown_delay
                );
                std::process::exit(1);
            }
        }
    }

    async fn shutdown_impl(&mut self) -> Result<(), ServerErr> {
        // the shutdown order is important here. The refresh and server threads rely on the
        // state so the state must be shutdown last.
        info!("Shutting down miru agent...");

        // 1. refresh
        if let Some(token_refresh_handle) = self.token_refresh_handle.take() {
            token_refresh_handle.await.map_err(|e| {
                ServerErr::JoinHandleErr(JoinHandleErr {
                    source: Box::new(e),
                    trace: trace!(),
                })
            })?;
        } else {
            info!("Token refresh handle not found, skipping token refresh shutdown...");
        }

        // 2. server
        if let Some(server_handle) = self.server_handle.take() {
            server_handle.await.map_err(|e| {
                ServerErr::JoinHandleErr(JoinHandleErr {
                    source: Box::new(e),
                    trace: trace!(),
                })
            })??;
        } else {
            info!("Server handle not found, skipping server shutdown...");
        }

        // 3. state
        if let Some(state_params) = self.state_params.take() {
            state_params.state.shutdown().await?;
            state_params.state_handle.await;
        } else {
            info!("State not found, skipping state shutdown...");
        }

        info!("Program shutdown complete");
        Ok(())
    }
}
