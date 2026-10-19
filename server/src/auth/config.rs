#[derive(Clone)]
pub struct Config {
    pub github_client_id: String,
    pub github_client_secret: String,
    pub github_callback_url: String,
    pub jwt_secret: String,
    pub frontend_url: String,
    pub base_url: String,
    pub agent_binary_path: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            github_client_id: env_or("GITHUB_CLIENT_ID", ""),
            github_client_secret: env_or("GITHUB_CLIENT_SECRET", ""),
            github_callback_url: env_or(
                "GITHUB_CALLBACK_URL",
                "http://localhost:8000/auth/github/callback",
            ),
            // En producción este valor debe venir del entorno, nunca usar el default.
            jwt_secret: env_or("JWT_SECRET", "dev-secret"),
            frontend_url: env_or("FRONTEND_URL", "http://localhost:5173"),
            base_url: env_or("BASE_URL", "http://127.0.0.1:8000"),
            agent_binary_path: env_or(
                "AGENT_BINARY_PATH",
                "../agent/installer/agent-v0.1.0-beta-linux-amd64",
            ),
        }
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}
