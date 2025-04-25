// standard library
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{SystemTime, Duration};
use std::sync::atomic::{AtomicU64, Ordering};

// internal crates
use crate::auth::refresh::run_token_refresh_loop;
use crate::server::serve::serve;
use crate::server::state::ServerState;
use crate::server::errors::{ServerErr, JoinHandleErr};
use crate::storage::layout::StorageLayout;
use crate::trace;

// external
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tracing::{info, error};

pub struct ServerComponents {
    pub state: Arc<ServerState>,
    pub state_handle: Pin<Box<dyn Future<Output = ()>>>,
    pub token_refresh_handle: JoinHandle<()>,
    pub server_handle: JoinHandle<Result<(), ServerErr>>,
}

pub struct RunServerOptions {
    pub layout: StorageLayout,
    pub idle_timeout: Duration,
    pub idle_timeout_poll_interval: Duration,
    pub max_shutdown_delay: Duration,
}

impl Default for RunServerOptions {
    fn default() -> Self {
        Self {
            layout: StorageLayout::default(),
            idle_timeout: Duration::from_secs(60),
            idle_timeout_poll_interval: Duration::from_secs(5),
            max_shutdown_delay: Duration::from_secs(30),
        }
    }
}

pub async fn run(options: RunServerOptions) -> Result<(), ServerErr> {
    // Create a single shutdown channel that all components will listen to
    let (shutdown_tx, _shutdown_rx): (tokio::sync::broadcast::Sender<()>, _) = tokio::sync::broadcast::channel(1);
    let shutdown_rx_refresh = shutdown_tx.subscribe();
    let shutdown_rx2_server = shutdown_tx.subscribe();

    // start the server
    let start_server_result = start_server(
        options.layout,
        shutdown_rx_refresh,
        shutdown_rx2_server,
    ).await?;

    // wait for ctrl-c or idle timeout to trigger a shutdown
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Ctrl-C received, shutting down...");
        }
        _ = await_idle_timeout(
            start_server_result.state.last_activity.clone(),
            options.idle_timeout,
            options.idle_timeout_poll_interval,
        ) => {
            info!("Idle timeout reached, shutting down...");
        }
    }

    // shutdown the server
    drop(shutdown_tx);
    match tokio::time::timeout(options.max_shutdown_delay, shutdown(start_server_result)).await {
        Ok(result) => result,
        Err(_) => {
            error!("Shutdown timed out after {:?}, forcing shutdown...", options.max_shutdown_delay);
            std::process::exit(1);
        }
    }
}

async fn start_server(
    layout: StorageLayout,
    mut shutdown_rx_refresh: broadcast::Receiver<()>,
    mut shutdown_rx2_server: broadcast::Receiver<()>,
) -> Result<ServerComponents, ServerErr> {
    // initialize the server state
    let (state, state_handle) = ServerState::new(layout).await?;
    let state = Arc::new(state);

    // initialize the token refresh loop
    let token_mngr_for_spawn = state.token_mngr.clone();
    let token_refresh_handle = tokio::spawn(async move {
        run_token_refresh_loop(
            token_mngr_for_spawn,
            Duration::from_secs(30),
            async move { let _ = shutdown_rx_refresh.recv().await; },
        ).await;
    });

    // run the server with graceful shutdown
    let server_handle = serve(
        state.clone(),
        async move { let _ = shutdown_rx2_server.recv().await; },
    ).await?;

    Ok(ServerComponents {
        state,
        state_handle: Box::pin(state_handle),
        token_refresh_handle,
        server_handle,
    })
}

async fn await_idle_timeout(
    shared_last_activity: Arc<AtomicU64>,
    idle_timeout: Duration,
    poll_interval: Duration,
) -> Result<(), ServerErr> {
    loop {
        info!("Checking server idle timeout...");
        tokio::time::sleep(poll_interval).await;
        let last_activity = SystemTime::UNIX_EPOCH + Duration::from_secs(
            shared_last_activity.load(Ordering::Relaxed)
        );
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

async fn shutdown(server_components: ServerComponents) -> Result<(), ServerErr> {
    // the shutdown order is important here. The refresh and server threads rely on the
    // state so the state must be shutdown last.
    info!("Shutting down program...");

    // 1. refresh
    let token_refresh_handle = server_components.token_refresh_handle;
    token_refresh_handle.await.map_err(|e| ServerErr::JoinHandleErr(JoinHandleErr {
        source: Box::new(e),
        trace: trace!(),
    }))?;

    // 2. server
    let server_handle = server_components.server_handle;
    server_handle.await.map_err(|e| ServerErr::JoinHandleErr(JoinHandleErr {
        source: Box::new(e),
        trace: trace!(),
    }))??;

    // 3. state
    let state = server_components.state;
    let state_handle = server_components.state_handle;
    state.shutdown().await?;
    state_handle.await;

    info!("Program shutdown complete");
    Ok(())
}