mod auth;
mod db;
mod enrollment;
mod routes;
mod state;
mod types;

use crate::state::AppState;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let pool = db::connect(&db::config::DbConfig::from_env())
        .await
        .unwrap_or_else(|e| panic!("failed to connect to database: {e}"));

    let state = AppState {
        pool,
        config: auth::Config::from_env(),
    };

    let app = routes::router(state);

    let addr = std::env::var("ADDR").unwrap_or_else(|_| routes::ADDR.to_string());
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("failed to bind {}: {e}", addr));

    println!("OctoOps backend listening on http://{addr}");

    axum::serve(listener, app)
        .await
        .expect("server error");
}
