// standard library
use std::future::Future;
use std::sync::Arc;
use std::time::{SystemTime, Duration};
use std::sync::atomic::Ordering;

// internal crates
use crate::auth::refresh::run_token_refresh_loop;
use crate::server::serve::serve;
use crate::server::state::ServerState;
use crate::server::errors::{ServerErr, JoinHandleErr};
use crate::storage::layout::StorageLayout;
use crate::trace;

// external
use tokio::task::JoinHandle;
use tracing::{info, error};


pub async fn run(layout: StorageLayout) -> Result<(), ServerErr> {
    // Create a single shutdown channel that all components will listen to
    let (shutdown_tx, _shutdown_rx): (tokio::sync::broadcast::Sender<()>, _) = tokio::sync::broadcast::channel(1);
    let mut shutdown_rx_refresh = shutdown_tx.subscribe();
    let mut shutdown_rx2_server = shutdown_tx.subscribe();

    // initialize the server state
    let (state, state_handle) = ServerState::new(layout).await?;
    let state = Arc::new(state);

    // initialize the token refresh loop to ensure the token is fresh for the duration
    // of the server's lifetime
    let token_mngr_for_spawn = state.token_mngr.clone();
    let refresh_handle = tokio::spawn(async move {
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

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Ctrl-C received, shutting down...");
        }
        _ = async {
            let idle_timeout = Duration::from_secs(10);
            loop {
                info!("Checking server idle timeout...");
                tokio::time::sleep(Duration::from_secs(1)).await;
                let last_activity = SystemTime::UNIX_EPOCH + Duration::from_secs(
                    state.last_activity.load(Ordering::Relaxed)
                );
                match SystemTime::now().duration_since(last_activity) {
                    Ok(duration) if duration > idle_timeout => {
                        info!("Server idle timeout reached, shutting down...");
                        break;
                    }
                    Err(_) => {
                        error!("Server idle timeout checker error, ignoring...");
                    }
                    _ => {}
                }
            }
        } => {
            info!("Idle timeout reached, shutting down...");
        }
    }

    drop(shutdown_tx);
    shutdown(
        refresh_handle,
        server_handle,
        state,
        state_handle,
    ).await?;

    Ok(())
}

async fn shutdown(
    refresh_handle: JoinHandle<()>,
    server_handle: JoinHandle<Result<(), ServerErr>>,
    state: Arc<ServerState>,
    state_handle: impl Future<Output = ()>,
) -> Result<(), ServerErr> {
    // the shutdown order is important here
    info!("Shutting down program...");

    // 1. refresh
    refresh_handle.await.map_err(|e| ServerErr::JoinHandleErr(JoinHandleErr {
        source: Box::new(e),
        trace: trace!(),
    }))?;

    // 2. server
    server_handle.await.map_err(|e| ServerErr::JoinHandleErr(JoinHandleErr {
        source: Box::new(e),
        trace: trace!(),
    }))??;

    // 3. state
    state.shutdown().await?;
    state_handle.await;
    info!("Program shutdown complete");
    Ok(())
}