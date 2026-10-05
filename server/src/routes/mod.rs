pub mod health;
pub mod hello;

use axum::Router;
use sqlx::PgPool;

pub use hello::ADDR;

pub fn router(db: PgPool) -> Router {
    Router::new()
        .merge(hello::router())
        .merge(health::router(db))
}
