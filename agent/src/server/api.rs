// std
use std::sync::Arc;

// internal crates
use crate::http_client::client::HTTPClient;
use crate::logs::{init, LogLevel};
use crate::server::handlers;
use crate::storage::layout::StorageLayout;
use crate::storage::digests::AsyncConfigSchemaDigestCache;

// external
use axum::{
    routing::{get, post},
    http::StatusCode,
    Json, Router,
};
use serde_json::json;

pub async fn server() {
    let result = init(true, LogLevel::Info);
    if let Err(e) = result {
        println!("Failed to initialize logging: {}", e);
    }

    // setup the http client
    let http_client = Arc::new(HTTPClient::new().await);

    // setup the caches
    let layout = StorageLayout::new_default();
    let dir = layout.cfg_sch_digest_registry();
    let cache = Arc::new(AsyncConfigSchemaDigestCache::spawn(dir));

    // build the app with the test route
    let app = Router::new()
        .route("/v1/test", get(test))

    // ============================ CONCRETE CONFIGS ============================== //
    // app.route(
    //     "/v1/concrete_configs/latest",
    //     get(read_latest_concrete_config),
    // );
    // app.route(
    //     "/v1/concrete_configs/refresh_latest",
    //     post(refresh_latest_concrete_config),
    // );

    // ============================ CONFIG SCHEMAS ============================== //
    .route(
        "/v1/config_schemas/hash",
        post({
            let cache = Arc::clone(&cache);
            move |payload| handlers::hash_schema(
                payload,
                http_client,
                cache,
            )
        }),
    );

    // run the server over the unix socket
    let socket_path = "/tmp/miru.sock";
    let _ = std::fs::remove_file(socket_path);
    let listener = tokio::net::UnixListener::bind(socket_path).unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn test() -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::OK, Json(json!({
        "status": "ok",
        "server": "axum"
    })))
}


