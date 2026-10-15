mod auth;
mod db;
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

    let listener = tokio::net::TcpListener::bind(routes::ADDR)
        .await
        .unwrap_or_else(|e| panic!("failed to bind {}: {e}", routes::ADDR));

    println!("OctoOps backend listening on http://{}", routes::ADDR);

    axum::serve(listener, app)
        .await
        .expect("server error");
}
