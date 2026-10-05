use axum::routing::get;
use axum::{Json, Router};

use crate::contracts::HelloResponse;

pub const ADDR: &str = "127.0.0.1:8000";

pub fn router() -> Router {
    Router::new().route("/", get(hello))
}

async fn hello() -> Json<HelloResponse> {
    Json(HelloResponse {
        message: "Hello from OctoOps backend".to_string(),
        version: env!("CARGO_PKG_VERSION"),
    })
}
