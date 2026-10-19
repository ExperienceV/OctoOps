use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::Router;
use uuid::Uuid;

use crate::db;
use crate::state::AppState;
use crate::types::Report;

const REPLAY_WINDOW_SECS: i64 = 30;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/servers/agents/report", post(report))
        .with_state(state)
}

async fn report(State(state): State<AppState>, headers: HeaderMap, body: Bytes) -> Response {
    let Some(agent_id) = header_str(&headers, "x-agent-id") else {
        return (StatusCode::BAD_REQUEST, "missing X-Agent-ID").into_response();
    };
    let Some(timestamp) = header_str(&headers, "x-timestamp") else {
        return (StatusCode::BAD_REQUEST, "missing X-Timestamp").into_response();
    };
    let Some(signature) = header_str(&headers, "x-signature") else {
        return (StatusCode::BAD_REQUEST, "missing X-Signature").into_response();
    };

    let Ok(server_id) = Uuid::parse_str(agent_id) else {
        return (StatusCode::UNAUTHORIZED, "invalid agent id").into_response();
    };

    let Ok(ts) = timestamp.parse::<i64>() else {
        return (StatusCode::BAD_REQUEST, "invalid timestamp").into_response();
    };
    if (now_secs() - ts).abs() > REPLAY_WINDOW_SECS {
        return (StatusCode::UNAUTHORIZED, "stale timestamp").into_response();
    }

    let server = match db::queries::get_server_for_auth(&state.pool, server_id).await {
        Ok(Some(server)) => server,
        Ok(None) => return (StatusCode::UNAUTHORIZED, "unknown agent").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response(),
    };

    let Some(public_key) = server.0 else {
        return (StatusCode::UNAUTHORIZED, "agent not registered").into_response();
    };
    if server.1 != "active" {
        return (StatusCode::UNAUTHORIZED, "agent not active").into_response();
    }

    let Ok(public_key_bytes) = decode_array::<32>(&public_key) else {
        return (StatusCode::UNAUTHORIZED, "invalid public key").into_response();
    };
    let Ok(signature_bytes) = decode_array::<64>(signature) else {
        return (StatusCode::BAD_REQUEST, "invalid signature").into_response();
    };

    let Ok(verifying_key) = ed25519_dalek::VerifyingKey::from_bytes(&public_key_bytes) else {
        return (StatusCode::UNAUTHORIZED, "invalid public key").into_response();
    };
    let signature = ed25519_dalek::Signature::from_bytes(&signature_bytes);

    // El agente firma: timestamp + "\n" + body
    let mut message = timestamp.as_bytes().to_vec();
    message.push(b'\n');
    message.extend_from_slice(&body);

    if verifying_key.verify_strict(&message, &signature).is_err() {
        return (StatusCode::UNAUTHORIZED, "invalid signature").into_response();
    }

    // Validar la forma del payload (number | bool en las hojas).
    if serde_json::from_slice::<Report>(body.as_ref()).is_err() {
        return (StatusCode::BAD_REQUEST, "invalid report payload").into_response();
    }

    let metrics_json = String::from_utf8_lossy(&body).to_string();
    if db::queries::insert_metric(&state.pool, server_id, &metrics_json)
        .await
        .is_err()
    {
        return (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response();
    }

    StatusCode::NO_CONTENT.into_response()
}

fn header_str<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name)?.to_str().ok()
}

fn decode_array<const N: usize>(hex: &str) -> Result<[u8; N], hex::FromHexError> {
    let bytes = hex::decode(hex)?;
    bytes
        .try_into()
        .map_err(|_| hex::FromHexError::InvalidStringLength)
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_secs() as i64
}
