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
