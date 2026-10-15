use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

pub const ACCESS_TTL_SECS: i64 = 30 * 60; // 30 minutos
pub const REFRESH_TTL_SECS: i64 = 7 * 24 * 60 * 60; // 7 días

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    #[serde(rename = "type")]
    pub typ: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,
    pub exp: usize,
}

pub fn sign_access(user_id: &str, secret: &str) -> String {
    let claims = Claims {
        sub: user_id.to_string(),
        typ: "access".to_string(),
        jti: None,
        exp: now_secs() + ACCESS_TTL_SECS as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .expect("jwt encode should not fail")
}

pub fn sign_refresh(user_id: &str, jti: &str, secret: &str) -> String {
    let claims = Claims {
        sub: user_id.to_string(),
        typ: "refresh".to_string(),
        jti: Some(jti.to_string()),
        exp: now_secs() + REFRESH_TTL_SECS as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .expect("jwt encode should not fail")
}

pub fn verify(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(jsonwebtoken::Algorithm::HS256),
    )?;
    Ok(data.claims)
}

fn now_secs() -> usize {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_secs() as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn access_roundtrip() {
        let token = sign_access("user-1", "secret");
        let claims = verify(&token, "secret").unwrap();
        assert_eq!(claims.sub, "user-1");
        assert_eq!(claims.typ, "access");
        assert!(claims.jti.is_none());
    }

    #[test]
    fn refresh_roundtrip() {
        let token = sign_refresh("user-1", "jti-1", "secret");
        let claims = verify(&token, "secret").unwrap();
        assert_eq!(claims.sub, "user-1");
        assert_eq!(claims.typ, "refresh");
        assert_eq!(claims.jti.as_deref(), Some("jti-1"));
    }

    #[test]
    fn wrong_secret_fails() {
        let token = sign_access("user-1", "secret");
        assert!(verify(&token, "wrong-secret").is_err());
    }
}
