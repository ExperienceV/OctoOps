use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};

use crate::state::AppState;
use crate::types::Report;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/servers/agents/report", post(report))
        .with_state(state)
}

async fn report(State(_state): State<AppState>, Json(payload): Json<Report>) -> StatusCode {
    println!(
        "report: {}",
        serde_json::to_string_pretty(&payload).unwrap_or_default()
    );
    StatusCode::NO_CONTENT
}
