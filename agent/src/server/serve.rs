// standard library
use std::future::Future;
use std::sync::Arc;

// internal crates
use crate::auth::refresh::run_token_refresh_loop;
use crate::server::handlers;
use crate::server::state::ServerState;
use crate::server::errors::{ServerErr, JoinHandleErr, IOErr, SendShutdownSignalErr};
use crate::storage::layout::StorageLayout;
use crate::trace;

// external
use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use tokio::task::JoinHandle;
use tokio::time::Duration;
use tokio::sync::oneshot;
use tower::ServiceBuilder;
use tower_http::{
    LatencyUnit,
    trace::{TraceLayer, DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse},
};
use tracing::Level;
use tracing::info;

pub async fn run(layout: StorageLayout) -> Result<(), ServerErr> {
    let (state, state_handle) = ServerState::new(layout).await?;
    let state = Arc::new(state);

    // run the token refresh loop to keep the token fresh for the duration of the
    // server's lifetime
    let token_mngr_for_spawn = state.token_mngr.clone();
    let (refresh_shutdown_tx, refresh_shutdown_rx) = oneshot::channel::<()>();
    let refresh_shutdown = async move {
        let _ = refresh_shutdown_rx.await;
        info!("Shutting down token refresh loop...");
    };
    let refresh_handle = run_token_refresh_loop(
        token_mngr_for_spawn,
        Duration::from_secs(60),
        refresh_shutdown,
    );

    // serve with a graceful shutdown
    let (server_shutdown_tx, server_shutdown_rx) = oneshot::channel::<()>();
    let server_shutdown= async move {
        let _ = server_shutdown_rx.await;
        info!("Shutting down server...");
    };
    let server_handle = serve(
        state.clone(),
        server_shutdown,
    ).await?;

    // wait 10 seconds and then shutdown everything (state, refresh, server)
    std::thread::sleep(Duration::from_secs(10));

    // send the shutdown signals
    // 1. state
    state.shutdown().await?;
    // 2. refresh
    refresh_shutdown_tx.send(()).map_err(|_| ServerErr::SendShutdownSignalErr(SendShutdownSignalErr {
        service: "token refresher".to_string(),
        trace: trace!(),
    }))?;
    // 3. server
    server_shutdown_tx.send(()).map_err(|_| ServerErr::SendShutdownSignalErr(SendShutdownSignalErr {
        service: "server".to_string(),
        trace: trace!(),
    }))?;

    // wait for the shutdown to complete
    // 1. state
    state_handle.await;
    // 2. refresh
    refresh_handle.await;
    // 3. server
    server_handle.await.map_err(|e| ServerErr::JoinHandleErr(JoinHandleErr {
        source: Box::new(e),
        trace: trace!(),
    }))??;

    Ok(())
}

pub async fn serve(
    state: Arc<ServerState>,
    shutdown_signal: impl Future<Output = ()> + Send + 'static,
) -> Result<JoinHandle<Result<(), ServerErr>>, ServerErr> {
    info!("Starting server...");

    // build the app with the test route
    let app = Router::new()
        .route("/v1/test", get(test))
        // ============================ CONCRETE CONFIGS ============================== //
        .route(
            "/v1/concrete_configs/latest",
            get(handlers::read_latest_concrete_config),
        )
        .route(
            "/v1/concrete_configs/refresh_latest",
            post(handlers::refresh_latest_concrete_config),
        )
        // ============================ CONFIG SCHEMAS ============================== //
        .route("/v1/config_schemas/hash", post(handlers::hash_schema))
        .layer(ServiceBuilder::new()
            .layer(TraceLayer::new_for_http()
                .make_span_with(
                DefaultMakeSpan::new().include_headers(true)
            )
            .on_request(
                DefaultOnRequest::new().level(Level::INFO)
            )
            .on_response(
                DefaultOnResponse::new()
                    .level(Level::INFO)
                    .latency_unit(LatencyUnit::Micros)
            )
        )
        )
        .with_state(state);

    // run the server over the unix socket
    let socket_path = "/tmp/miru.sock";
    let _ = std::fs::remove_file(socket_path);
    let listener = tokio::net::UnixListener::bind(socket_path).map_err(|e| ServerErr::IOErr(IOErr {
        source: e,
        trace: trace!(),
    }))?;

    // serve with graceful shutdown
    let server_handle =  tokio::task::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal)
            .await
            .map_err(|e| ServerErr::IOErr(IOErr {
                source: e,
                trace: trace!(),
            }))
    });

    Ok(server_handle)
}

async fn test() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "server": "axum"
        })),
    )
}
