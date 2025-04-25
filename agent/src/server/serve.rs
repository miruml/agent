// standard library
use std::future::Future;
use std::sync::Arc;

// internal crates
use crate::server::handlers;
use crate::server::state::ServerState;
use crate::server::errors::{ServerErr, IOErr};
use crate::trace;

// external
use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use tokio::task::JoinHandle;
use tower::ServiceBuilder;
use tower_http::{
    LatencyUnit,
    trace::{TraceLayer, DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse},
};
use tracing::Level;
use tracing::info;


pub async fn serve(
    state: Arc<ServerState>,
    shutdown_signal: impl Future<Output = ()> + Send + 'static,
) -> Result<JoinHandle<Result<(), ServerErr>>, ServerErr> {
    info!("Starting server...");

    // build the app with the test route
    let state_for_middleware = state.clone();
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
            // activity middleware
            .layer(axum::middleware::from_fn(move |req: axum::extract::Request, next: axum::middleware::Next| {
                let state = state_for_middleware.clone();
                async move {
                    state.update_last_activity();
                    next.run(req).await
                }
            }))
            // logging middleware
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
