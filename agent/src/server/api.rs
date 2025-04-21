// standard library
use std::sync::Arc;

// internal crates
use crate::http::client::HTTPClient;
use crate::logs::{init, LogLevel};
use crate::server::handlers;
use crate::storage::concrete_configs::ConcreteConfigCache;
use crate::storage::digests::ConfigSchemaDigestCache;
use crate::storage::layout::StorageLayout;

// external
use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use tower::ServiceBuilder;
use tower_http::{
    LatencyUnit,
    trace::{TraceLayer, DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse},
};
use tracing::Level;
use tracing::info;
#[derive(Clone)]
pub struct AppState {
    pub http_client: Arc<HTTPClient>,
    pub config_schema_digest_cache: Arc<ConfigSchemaDigestCache>,
    pub concrete_config_cache: Arc<ConcreteConfigCache>,
}

pub async fn server() {
    let result = init(true, LogLevel::Debug);
    if let Err(e) = result {
        println!("Failed to initialize logging: {}", e);
    }

    // setup the http client
    let layout = StorageLayout::new_default();
    let shared_state = Arc::new(AppState {
        http_client: Arc::new(HTTPClient::new().await),
        config_schema_digest_cache: Arc::new(ConfigSchemaDigestCache::spawn(
            layout.config_schema_digest_cache(),
        )),
        concrete_config_cache: Arc::new(ConcreteConfigCache::spawn(layout.concrete_config_cache())),
    });

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
        .with_state(shared_state);

    // run the server over the unix socket
    let socket_path = "/tmp/miru.sock";
    let _ = std::fs::remove_file(socket_path);
    let listener = tokio::net::UnixListener::bind(socket_path).unwrap();
    axum::serve(listener, app).await.unwrap();
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
