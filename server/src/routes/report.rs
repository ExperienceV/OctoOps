use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use sqlx::PgPool;

use crate::types::Report;

pub fn router(db: PgPool) -> Router {
    Router::new()
        .route("/servers/agents/report", post(report))
        .with_state(db)
}

async fn report(State(_db): State<PgPool>, Json(payload): Json<Report>) -> StatusCode {
    println!(
        "report: {}",
        serde_json::to_string_pretty(&payload).unwrap_or_default()
    );
    StatusCode::NO_CONTENT
}
