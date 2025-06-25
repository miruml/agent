// standard library
use std::future::Future;
use std::sync::Arc;
use std::{
    env,
    os::unix::io::{FromRawFd, RawFd},
};

// internal crates
use crate::filesys::{file::File, path::PathExt};
use crate::server::errors::{BindUnixSocketErr, RunAxumServerErr, ServerErr, ServerFileSysErr};
use crate::server::handlers;
use crate::server::state::ServerState;
use crate::trace;
use crate::utils::version_info;

// external
use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use tokio::net::UnixListener;
use tokio::task::JoinHandle;
use tower::ServiceBuilder;
use tower_http::{
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::Level;

pub(crate) async fn serve(
    socket_file: &File,
    state: Arc<ServerState>,
    shutdown_signal: impl Future<Output = ()> + Send + 'static,
) -> Result<JoinHandle<Result<(), ServerErr>>, ServerErr> {
    // build the app with the test route
    let state_for_middleware = state.clone();
    let app = Router::new()
        .route("/v1/test", get(test))
        .route("/v1/version", get(version))
        // ============================ CONFIG INSTANCES ============================== //
        .route(
            "/v1/config_instances/deployed",
            get(handlers::read_deployed_config_instance),
        )
        // ============================ CONFIG SCHEMAS ============================== //
        .route(
            "/v1/config_schemas/hash/serialized",
            post(handlers::hash_schema),
        )
        .layer(
            ServiceBuilder::new()
                // activity middleware
                .layer(axum::middleware::from_fn(
                    move |req: axum::extract::Request, next: axum::middleware::Next| {
                        let state = state_for_middleware.clone();
                        async move {
                            state.record_activity();
                            next.run(req).await
                        }
                    },
                ))
                // logging middleware
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::new().include_headers(true))
                        .on_request(DefaultOnRequest::new().level(Level::INFO))
                        .on_response(
                            DefaultOnResponse::new()
                                .level(Level::INFO)
                                .latency_unit(LatencyUnit::Micros),
                        ),
                ),
        )
        .with_state(state);

    // obtain the unix socket file listener
    let listener = acquire_unix_socket_listener(socket_file, async move {
        create_unix_socket_listener(socket_file).await
    })
    .await?;

    // serve with graceful shutdown
    let server_handle = tokio::task::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal)
            .await
            .map_err(|e| {
                ServerErr::RunAxumServerErr(RunAxumServerErr {
                    source: Box::new(e),
                    trace: trace!(),
                })
            })
    });

    Ok(server_handle)
}

async fn acquire_unix_socket_listener(
    socket_file: &File,
    fallback: impl Future<Output = Result<UnixListener, ServerErr>>,
) -> Result<UnixListener, ServerErr> {
    let listener = if let Ok(listen_fds) = env::var("LISTEN_FDS") {
        let listen_fds = listen_fds.parse::<u32>().map_err(|e| {
            ServerErr::BindUnixSocketErr(BindUnixSocketErr {
                socket_file: socket_file.clone(),
                source: Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to parse LISTEN_FDS: {}", e),
                )),
                trace: trace!(),
            })
        })?;
        if listen_fds >= 1 {
            // FD#3 is the first one
            let fd: RawFd = 3;
            // SAFETY: fd=3 was handed to us by systemd
            let std_listener = unsafe { std::os::unix::net::UnixListener::from_raw_fd(fd) };
            std_listener.set_nonblocking(true).map_err(|e| {
                ServerErr::BindUnixSocketErr(BindUnixSocketErr {
                    socket_file: socket_file.clone(),
                    source: Box::new(e),
                    trace: trace!(),
                })
            })?;
            UnixListener::from_std(std_listener).map_err(|e| {
                ServerErr::BindUnixSocketErr(BindUnixSocketErr {
                    socket_file: socket_file.clone(),
                    source: Box::new(e),
                    trace: trace!(),
                })
            })?
        } else {
            fallback.await?
        }
    } else {
        fallback.await?
    };
    Ok(listener)
}

async fn create_unix_socket_listener(socket_file: &File) -> Result<UnixListener, ServerErr> {
    socket_file.delete().await.map_err(|e| {
        ServerErr::FileSysErr(ServerFileSysErr {
            source: Box::new(e),
            trace: trace!(),
        })
    })?;
    let socket_path = socket_file.path();
    tokio::net::UnixListener::bind(socket_path).map_err(|e| {
        ServerErr::BindUnixSocketErr(BindUnixSocketErr {
            socket_file: socket_file.clone(),
            source: Box::new(e),
            trace: trace!(),
        })
    })
}

async fn version() -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::OK, Json(version_info()))
}

async fn test() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "server": "miru-config-agent"
        })),
    )
}
