use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GitHubUser {
    pub id: i64,
    pub login: String,
    pub avatar_url: String,
    pub email: Option<String>,
}

pub async fn exchange_code(
    code: &str,
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
) -> Result<String, String> {
    let client = reqwest::Client::new();
    let resp = client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("code", code),
            ("redirect_uri", redirect_uri),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let data: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    data["access_token"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "no access_token in github response".to_string())
}

pub async fn fetch_user(access_token: &str) -> Result<GitHubUser, String> {
    let client = reqwest::Client::new();
    client
        .get("https://api.github.com/user")
        .bearer_auth(access_token)
        .header("User-Agent", "octoops")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<GitHubUser>()
        .await
        .map_err(|e| e.to_string())
}
