use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use sqlx::PgPool;

use crate::contracts::HealthResponse;
use crate::db::queries;

pub fn router(db: PgPool) -> Router {
    Router::new().route("/health", get(health).with_state(db))
}

async fn health(State(pool): State<PgPool>) -> Json<HealthResponse> {
    let database = match queries::health::ping(&pool).await {
        Ok(()) => "up",
        Err(_) => "down",
    };

    Json(HealthResponse {
        status: "ok",
        database,
    })
}
