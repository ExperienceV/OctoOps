use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct HelloResponse {
    pub message: String,
    pub version: &'static str,
}
