use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct HealthResponse {
    pub status: &'static str,
    pub database: &'static str,
}
