use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub mod config;
pub mod queries;

use crate::db::config::DbConfig;

pub async fn connect(cfg: &DbConfig) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&cfg.url())
        .await
}
