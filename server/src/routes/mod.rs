pub mod auth;
pub mod report;
pub mod servers;

use axum::Router;

use crate::state::AppState;

pub const ADDR: &str = "127.0.0.1:8000";

pub fn router(state: AppState) -> Router {
    Router::new()
        .merge(report::router(state.clone()))
        .merge(auth::router(state.clone()))
        .merge(servers::router(state.clone()))
}
