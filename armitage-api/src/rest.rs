//! REST API implementation

use axum::{
    routing::get,
    Router,
};

pub fn create_rest_app() -> Router {
    Router::new()
        .route("/api/v1/health", get(health_check))
        .route("/api/v1/hosts", get(list_hosts))
        .route("/api/v1/services", get(list_services))
        .route("/api/v1/sessions", get(list_sessions))
        .route("/api/v1/credentials", get(list_credentials))
}

async fn health_check() -> &'static str {
    "OK"
}

async fn list_hosts() -> &'static str {
    "{\"hosts\": []}"
}

async fn list_services() -> &'static str {
    "{\"services\": []}"
}

async fn list_sessions() -> &'static str {
    "{\"sessions\": []}"
}

async fn list_credentials() -> &'static str {
    "{\"credentials\": []}"
}
