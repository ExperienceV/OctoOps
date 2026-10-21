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

/// Migraciones idempotentes del schema. Se ejecutan en cada arranque para mantener
/// la DB al día sin borrar datos (a diferencia de `schema.sql`, que solo aplica en la
/// primera inicialización). Agregar aquí cada cambio de schema futuro.
pub async fn migrate(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("ALTER TABLE servers ADD COLUMN IF NOT EXISTS kind text NOT NULL DEFAULT 'server'")
        .execute(pool)
        .await?;
    sqlx::query("ALTER TABLE servers ADD COLUMN IF NOT EXISTS token_hash text")
        .execute(pool)
        .await?;

    Ok(())
}
