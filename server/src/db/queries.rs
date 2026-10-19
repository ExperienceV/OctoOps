use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::github::GitHubUser;
use crate::types::User;

pub async fn upsert_user(pool: &PgPool, github: &GitHubUser) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();
    let github_user_id = github.id.to_string();

    let row = sqlx::query_as::<_, (Uuid,)>(
        r#"
        INSERT INTO users (id, github_user_id, username, email, avatar_url)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (github_user_id) DO UPDATE
        SET username = EXCLUDED.username,
            email = EXCLUDED.email,
            avatar_url = EXCLUDED.avatar_url
        RETURNING id
        "#,
    )
    .bind(id)
    .bind(&github_user_id)
    .bind(&github.login)
    .bind(&github.email)
    .bind(&github.avatar_url)
    .fetch_one(pool)
    .await?;

    Ok(row.0)
}

pub async fn insert_refresh_token(
    pool: &PgPool,
    id: Uuid,
    user_id: Uuid,
    ttl_secs: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (id, user_id, expires_at)
        VALUES ($1, $2, now() + make_interval(secs => $3))
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(ttl_secs as f64)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn revoke_refresh_token(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let row = sqlx::query(
        "DELETE FROM refresh_tokens WHERE id = $1 AND expires_at > now() RETURNING id",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row.is_some())
}

pub async fn get_user(pool: &PgPool, id: Uuid) -> Result<Option<User>, sqlx::Error> {
    let row = sqlx::query_as::<_, (Uuid, String, Option<String>, Option<String>)>(
        "SELECT id, username, email, avatar_url FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(id, username, email, avatar_url)| User {
        id,
        username,
        email,
        avatar_url,
    }))
}

pub async fn create_server(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
    token_hash: &str,
    ttl_secs: i64,
) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO servers (id, user_id, name, status, enrollment_token_hash, enrollment_expires_at)
        VALUES ($1, $2, $3, 'pending', $4, now() + make_interval(secs => $5))
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(name)
    .bind(token_hash)
    .bind(ttl_secs as f64)
    .execute(pool)
    .await?;

    Ok(id)
}

pub struct PendingServer {
    pub id: Uuid,
    pub name: String,
}

pub async fn get_pending_server_by_token_hash(
    pool: &PgPool,
    token_hash: &str,
) -> Result<Option<PendingServer>, sqlx::Error> {
    let row = sqlx::query_as::<_, (Uuid, String)>(
        r#"
        SELECT id, name FROM servers
        WHERE enrollment_token_hash = $1
          AND status = 'pending'
          AND enrollment_expires_at > now()
        "#,
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(id, name)| PendingServer { id, name }))
}

pub async fn activate_server(
    pool: &PgPool,
    id: Uuid,
    public_key: &str,
) -> Result<bool, sqlx::Error> {
    let row = sqlx::query(
        r#"
        UPDATE servers
        SET status = 'active',
            public_key = $2,
            enrollment_token_hash = NULL,
            enrollment_expires_at = NULL
        WHERE id = $1 AND status = 'pending'
        "#,
    )
    .bind(id)
    .bind(public_key)
    .execute(pool)
    .await?;

    Ok(row.rows_affected() == 1)
}

/// Devuelve `(public_key, status)` del server, para verificar reportes.
pub async fn get_server_for_auth(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<(Option<String>, String)>, sqlx::Error> {
    sqlx::query_as::<_, (Option<String>, String)>(
        "SELECT public_key, status FROM servers WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Inserta un reporte de métricas y actualiza `last_seen_at`.
pub async fn insert_metric(
    pool: &PgPool,
    server_id: Uuid,
    metrics_json: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO metrics (server_id, metrics) VALUES ($1, $2::jsonb)")
        .bind(server_id)
        .bind(metrics_json)
        .execute(pool)
        .await?;

    sqlx::query("UPDATE servers SET last_seen_at = now() WHERE id = $1")
        .bind(server_id)
        .execute(pool)
        .await?;

    Ok(())
}
