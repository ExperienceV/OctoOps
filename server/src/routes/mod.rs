pub mod report;

use axum::Router;
use sqlx::PgPool;

pub const ADDR: &str = "127.0.0.1:8000";

pub fn router(db: PgPool) -> Router {
    Router::new().merge(report::router(db))
}
