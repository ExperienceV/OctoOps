use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use url::Url;
use uuid::Uuid;

use crate::auth::{cookies, github, jwt};
use crate::db;
use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/auth/github/login", get(login))
        .route("/auth/github/callback", get(callback))
        .route("/auth/refresh", post(refresh))
        .route("/auth/me", get(me))
        .route("/auth/logout", post(logout))
        .with_state(state)
}

async fn login(State(state): State<AppState>) -> Redirect {
    let mut url = Url::parse("https://github.com/login/oauth/authorize").unwrap();
    url.query_pairs_mut()
        .append_pair("client_id", &state.config.github_client_id)
        .append_pair("redirect_uri", &state.config.github_callback_url)
        .append_pair("scope", "read:user user:email");
    Redirect::to(url.as_str())
}

#[derive(Deserialize)]
struct CallbackParams {
    code: String,
}

async fn callback(
    State(state): State<AppState>,
    Query(params): Query<CallbackParams>,
) -> Response {
    let access_token = match github::exchange_code(
        &params.code,
        &state.config.github_client_id,
        &state.config.github_client_secret,
        &state.config.github_callback_url,
    )
    .await
    {
        Ok(token) => token,
        Err(_) => return (StatusCode::UNAUTHORIZED, "github code exchange failed").into_response(),
    };

    let gh_user = match github::fetch_user(&access_token).await {
        Ok(user) => user,
        Err(_) => return (StatusCode::UNAUTHORIZED, "github fetch user failed").into_response(),
    };

    let user_id = match db::queries::upsert_user(&state.pool, &gh_user).await {
        Ok(id) => id,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let user_id_str = user_id.to_string();
    let access = jwt::sign_access(&user_id_str, &state.config.jwt_secret);

    let refresh_id = Uuid::new_v4();
    let refresh = jwt::sign_refresh(
        &user_id_str,
        &refresh_id.to_string(),
        &state.config.jwt_secret,
    );

    if let Err(e) = db::queries::insert_refresh_token(
        &state.pool,
        refresh_id,
        user_id,
        jwt::REFRESH_TTL_SECS,
    )
    .await
    {
        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }

    let mut response = Redirect::to(state.config.frontend_url.as_str()).into_response();
    cookies::set_tokens(response.headers_mut(), access, refresh);
    response
}

async fn refresh(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let token = match cookies::get(&headers, cookies::REFRESH_COOKIE) {
        Some(t) => t,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    let claims = match jwt::verify(&token, &state.config.jwt_secret) {
        Ok(c) if c.typ == "refresh" => c,
        _ => return StatusCode::UNAUTHORIZED.into_response(),
    };

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
    };

    let refresh_id = match claims.jti.as_deref().and_then(|j| Uuid::parse_str(j).ok()) {
        Some(id) => id,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    let revoked = match db::queries::revoke_refresh_token(&state.pool, refresh_id).await {
        Ok(revoked) => revoked,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    if !revoked {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let user_id_str = user_id.to_string();
    let access = jwt::sign_access(&user_id_str, &state.config.jwt_secret);

    let new_refresh_id = Uuid::new_v4();
    let refresh = jwt::sign_refresh(
        &user_id_str,
        &new_refresh_id.to_string(),
        &state.config.jwt_secret,
    );

    if let Err(_) = db::queries::insert_refresh_token(
        &state.pool,
        new_refresh_id,
        user_id,
        jwt::REFRESH_TTL_SECS,
    )
    .await
    {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let mut response = StatusCode::NO_CONTENT.into_response();
    cookies::set_tokens(response.headers_mut(), access, refresh);
    response
}

async fn me(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let token = match cookies::get(&headers, cookies::ACCESS_COOKIE) {
        Some(t) => t,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    let claims = match jwt::verify(&token, &state.config.jwt_secret) {
        Ok(c) if c.typ == "access" => c,
        _ => return StatusCode::UNAUTHORIZED.into_response(),
    };

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
    };

    match db::queries::get_user(&state.pool, user_id).await {
        Ok(Some(user)) => Json(user).into_response(),
        Ok(None) => StatusCode::UNAUTHORIZED.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn logout(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Some(token) = cookies::get(&headers, cookies::REFRESH_COOKIE) {
        if let Ok(claims) = jwt::verify(&token, &state.config.jwt_secret) {
            if claims.typ == "refresh" {
                if let Some(jti) = claims.jti.as_deref() {
                    if let Ok(id) = Uuid::parse_str(jti) {
                        let _ = db::queries::revoke_refresh_token(&state.pool, id).await;
                    }
                }
            }
        }
    }

    let mut response = StatusCode::NO_CONTENT.into_response();
    cookies::clear_tokens(response.headers_mut());
    response
}
