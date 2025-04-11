// external
use axum::{
    routing::{get, post},
    http::StatusCode,
    Json, Router,
    extract::Query,
};
use serde::Deserialize;
use serde_json::json;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        .route("/v1/test", get(test))
        .route("/v1/concrete_configs/latest", get(get_latest_concrete_config))
        .route("/v1/config_schemas/hash", post(hash_schema));

    // run our app with hyper, listening globally on port 3000
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

async fn hash_schema() -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::OK, Json(json!({
        "digest": "1234567890"
    })))
}

#[derive(Debug, Deserialize)]
struct GetLatestConcreteConfigParams {
    config_schema_digest: String,
    config_slug: String,
}

async fn get_latest_concrete_config(
    Query(_): Query<GetLatestConcreteConfigParams>,
) -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::OK, Json(json!({
        "object": "concrete_config",
        "id": "cncr_cfg_123",
        "created_at": "2021-01-01T00:00:00Z",
        "created_by_id": "usr_123",
        "client_id": "cli_123",
        "config_schema_id": "cfg_sch_123",
        "concrete_config": {
            "device_id": "device_23jt0321p9123434gsdf",
            "speed": 100
        }
    })))
}
