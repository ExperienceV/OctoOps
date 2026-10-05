mod contracts;
mod db;
mod routes;

#[tokio::main]
async fn main() {
    let pool = db::connect(&db::config::DbConfig::from_env())
        .await
        .unwrap_or_else(|e| panic!("failed to connect to database: {e}"));

    let app = routes::router(pool);

    let listener = tokio::net::TcpListener::bind(routes::ADDR)
        .await
        .unwrap_or_else(|e| panic!("failed to bind {}: {e}", routes::ADDR));

    println!("OctoOps backend listening on http://{}", routes::ADDR);

    axum::serve(listener, app)
        .await
        .expect("server error");
}
