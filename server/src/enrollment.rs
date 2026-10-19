use sha2::{Digest, Sha256};

/// Validez del token de enrollment (24 horas).
pub const TTL_SECS: i64 = 24 * 60 * 60;

/// Genera un token de enrollment aleatorio (32 bytes, hex).
pub fn generate() -> String {
    let bytes: [u8; 32] = rand::random();
    hex::encode(bytes)
}

/// Hash SHA-256 del token. Solo esto se guarda en la DB.
pub fn hash(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}
