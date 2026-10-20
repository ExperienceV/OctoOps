use axum::extract::{Path, Query, State};
use axum::http::header::{HeaderMap, HeaderValue};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::cookies;
use crate::db;
use crate::enrollment;
use crate::state::AppState;

/// Factor de gracia para marcar un servidor como `offline`:
/// `now - last_seen > interval * 3`.
const OFFLINE_GRACE_FACTOR: i64 = 3;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/servers", get(list_servers).post(create_server))
        .route("/servers/install/bin", get(install_bin))
        .route("/servers/install/{token}", get(install_script))
        .route("/servers/agents/register", post(register))
        .route("/servers/{id}", get(get_server))
        .route("/servers/{id}/metrics", get(list_metrics))
        .with_state(state)
}

#[derive(Serialize)]
struct ServerView {
    id: Uuid,
    name: String,
    status: &'static str,
    interval_seconds: Option<i32>,
    last_seen_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

#[derive(Serialize)]
struct MetricView {
    recorded_at: DateTime<Utc>,
    metrics: serde_json::Value,
}

fn server_status(interval_seconds: Option<i32>, last_seen_at: Option<DateTime<Utc>>) -> &'static str {
    match last_seen_at {
        None => "unreported",
        Some(seen) => {
            let interval = interval_seconds.unwrap_or(10).max(1) as i64;
            let grace = chrono::Duration::seconds(interval * OFFLINE_GRACE_FACTOR);
            if Utc::now() - seen <= grace {
                "online"
            } else {
                "offline"
            }
        }
    }
}

fn into_view(row: db::queries::ServerRow) -> ServerView {
    ServerView {
        id: row.id,
        name: row.name,
        status: server_status(row.interval_seconds, row.last_seen_at),
        interval_seconds: row.interval_seconds,
        last_seen_at: row.last_seen_at,
        created_at: row.created_at,
    }
}

async fn list_servers(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let Some(user_id) = cookies::current_user_id(&headers, &state.config.jwt_secret) else {
        return (StatusCode::UNAUTHORIZED, "not authenticated").into_response();
    };

    match db::queries::list_servers(&state.pool, user_id).await {
        Ok(servers) => {
            let views: Vec<ServerView> = servers.into_iter().map(into_view).collect();
            Json(views).into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response(),
    }
}

async fn get_server(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Response {
    let Some(user_id) = cookies::current_user_id(&headers, &state.config.jwt_secret) else {
        return (StatusCode::UNAUTHORIZED, "not authenticated").into_response();
    };

    match db::queries::get_server(&state.pool, user_id, id).await {
        Ok(Some(server)) => Json(into_view(server)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "server not found").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response(),
    }
}

#[derive(Deserialize)]
struct MetricsParams {
    limit: Option<i64>,
}

async fn list_metrics(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Query(params): Query<MetricsParams>,
) -> Response {
    let Some(user_id) = cookies::current_user_id(&headers, &state.config.jwt_secret) else {
        return (StatusCode::UNAUTHORIZED, "not authenticated").into_response();
    };

    let owned = match db::queries::get_server(&state.pool, user_id, id).await {
        Ok(Some(_)) => true,
        Ok(None) => return (StatusCode::NOT_FOUND, "server not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response(),
    };
    if !owned {
        return (StatusCode::NOT_FOUND, "server not found").into_response();
    }

    let limit = params.limit.unwrap_or(100).clamp(1, 1000);

    match db::queries::list_metrics(&state.pool, id, limit).await {
        Ok(metrics) => {
            let views: Vec<MetricView> = metrics
                .into_iter()
                .map(|m| MetricView {
                    recorded_at: m.recorded_at,
                    metrics: m.metrics,
                })
                .collect();
            Json(views).into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response(),
    }
}

#[derive(Deserialize)]
struct CreateServerBody {
    name: String,
}

#[derive(Serialize)]
struct CreateServerResponse {
    install_command: String,
}

async fn create_server(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateServerBody>,
) -> Response {
    let name = body.name.trim();
    if name.is_empty() {
        return (StatusCode::BAD_REQUEST, "name is required").into_response();
    }

    let Some(user_id) = cookies::current_user_id(&headers, &state.config.jwt_secret) else {
        return (StatusCode::UNAUTHORIZED, "not authenticated").into_response();
    };

    let token = enrollment::generate();
    let token_hash = enrollment::hash(&token);

    if let Err(_) = db::queries::create_server(
        &state.pool,
        user_id,
        name,
        &token_hash,
        enrollment::TTL_SECS,
    )
    .await
    {
        return (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response();
    }

    let install_command = format!("curl -fsSL {}/servers/install/{} | sh", state.config.base_url, token);
    Json(CreateServerResponse { install_command }).into_response()
}

async fn install_script(State(state): State<AppState>, Path(token): Path<String>) -> Response {
    let token_hash = enrollment::hash(&token);

    let server = match db::queries::get_pending_server_by_token_hash(&state.pool, &token_hash).await
    {
        Ok(Some(server)) => server,
        Ok(None) => return (StatusCode::NOT_FOUND, "invalid or expired token").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response(),
    };

    let script = install_script_content(&state.config.base_url, &token, &server.name);
    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, HeaderValue::from_static("text/x-shellscript"))],
        script,
    )
        .into_response()
}

fn install_script_content(base_url: &str, token: &str, name: &str) -> String {
    let config = serde_json::json!({
        "baseUrl": base_url,
        "intervalSeconds": 10,
        "token": token,
        "name": name,
        "routes": {
            "metrics": { "path": "/servers/agents/report", "method": "POST" },
            "register": { "path": "/servers/agents/register", "method": "POST" }
        }
    });
    let config = serde_json::to_string(&config).expect("config json should serialize");

    format!(
        "#!/bin/sh\n\
         set -e\n\
         \n\
         mkdir -p OctoAgent\n\
         cd OctoAgent\n\
         \n\
         echo 'Descargando binario del agente...'\n\
         curl -fsSL '{base_url}/servers/install/bin' -o octo-agent\n\
         chmod +x octo-agent\n\
         \n\
         cat > config.json <<'EOF'\n\
         {config}\n\
         EOF\n\
         \n\
         echo 'Instalado en OctoAgent/. Ejecuta: ./OctoAgent/octo-agent'\n",
        base_url = base_url,
        config = config,
    )
}

async fn install_bin(State(state): State<AppState>) -> Response {
    match tokio::fs::read(&state.config.agent_binary_path).await {
        Ok(bytes) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                axum::http::header::CONTENT_TYPE,
                HeaderValue::from_static("application/octet-stream"),
            );
            (headers, bytes).into_response()
        }
        Err(_) => (StatusCode::NOT_FOUND, "agent binary not found").into_response(),
    }
}

#[derive(Deserialize)]
struct RegisterBody {
    token: String,
    public_key: String,
    interval_seconds: Option<i32>,
}

#[derive(Serialize)]
struct RegisterResponse {
    server_id: Uuid,
}

async fn register(State(state): State<AppState>, Json(body): Json<RegisterBody>) -> Response {
    if body.public_key.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "public_key is required").into_response();
    }

    let token_hash = enrollment::hash(&body.token);

    let server = match db::queries::get_pending_server_by_token_hash(&state.pool, &token_hash).await
    {
        Ok(Some(server)) => server,
        Ok(None) => return (StatusCode::UNAUTHORIZED, "invalid or expired token").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response(),
    };

    let interval_seconds = body.interval_seconds.unwrap_or(10).max(1);

    match db::queries::activate_server(&state.pool, server.id, &body.public_key, interval_seconds).await
    {
        Ok(true) => Json(RegisterResponse { server_id: server.id }).into_response(),
        Ok(false) => (StatusCode::CONFLICT, "server already registered").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response(),
    }
}
