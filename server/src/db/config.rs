pub struct DbConfig {
    host: String,
    port: u16,
    user: String,
    password: String,
    dbname: String,
}

impl DbConfig {
    pub fn from_env() -> Self {
        Self {
            host: env_or("PGHOST", "127.0.0.1"),
            port: env_or("PGPORT", "5432").parse().unwrap_or(5432),
            user: env_or("PGUSER", "octoops"),
            password: env_or("PGPASSWORD", "octoops"),
            dbname: env_or("PGDATABASE", "octoops"),
        }
    }

    pub fn url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.dbname
        )
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}
