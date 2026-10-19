use axum::http::header::{HeaderMap, HeaderValue, COOKIE, SET_COOKIE};
use uuid::Uuid;

use crate::auth::jwt::{self, ACCESS_TTL_SECS, REFRESH_TTL_SECS};

pub const ACCESS_COOKIE: &str = "access_token";
pub const REFRESH_COOKIE: &str = "refresh_token";

pub fn get(headers: &HeaderMap, name: &str) -> Option<String> {
    let header = headers.get(COOKIE)?.to_str().ok()?;
    header.split(';').find_map(|pair| {
        let (key, value) = pair.trim().split_once('=')?;
        (key == name).then(|| value.to_string())
    })
}

pub fn set_tokens(headers: &mut HeaderMap, access: String, refresh: String) {
    headers.append(SET_COOKIE, cookie(ACCESS_COOKIE, &access, ACCESS_TTL_SECS));
    headers.append(SET_COOKIE, cookie(REFRESH_COOKIE, &refresh, REFRESH_TTL_SECS));
}

pub fn clear_tokens(headers: &mut HeaderMap) {
    headers.append(SET_COOKIE, cookie(ACCESS_COOKIE, "", 0));
    headers.append(SET_COOKIE, cookie(REFRESH_COOKIE, "", 0));
}

/// Devuelve el `user_id` del access token de la cookie, o `None` si no es válido.
pub fn current_user_id(headers: &HeaderMap, secret: &str) -> Option<Uuid> {
    let token = get(headers, ACCESS_COOKIE)?;
    let claims = jwt::verify(&token, secret).ok()?;
    if claims.typ != "access" {
        return None;
    }
    Uuid::parse_str(&claims.sub).ok()
}

fn cookie(name: &str, value: &str, max_age_secs: i64) -> HeaderValue {
    HeaderValue::from_str(&format!(
        "{name}={value}; HttpOnly; SameSite=Lax; Path=/; Max-Age={max_age_secs}"
    ))
    .expect("cookie header should be valid")
}
