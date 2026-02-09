use security_framework::passwords::get_generic_password;
use serde::Deserialize;

#[derive(Deserialize)]
struct KeychainCredentials {
    #[serde(rename = "claudeAiOauth")]
    claude_ai_oauth: Option<OAuthCredentials>,
}

#[derive(Deserialize)]
struct OAuthCredentials {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "refreshToken")]
    refresh_token: String,
    #[allow(dead_code)]
    #[serde(rename = "expiresAt")]
    expires_at: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    #[allow(dead_code)]
    expires_in: Option<u64>,
}

pub struct Tokens {
    pub access_token: String,
    pub refresh_token: String,
}

const SERVICE: &str = "Claude Code-credentials";
const CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";
const TOKEN_URL: &str = "https://console.anthropic.com/v1/oauth/token";

fn get_account() -> Result<String, String> {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .map_err(|_| "Could not determine current username from USER or USERNAME env var".to_string())
}

pub fn read_keychain() -> Result<Tokens, String> {
    let account = get_account()?;
    let password = get_generic_password(SERVICE, &account)
        .map_err(|e| format!("Failed to read keychain (service={SERVICE:?}, account={account:?}): {e}"))?;

    let json_str =
        std::str::from_utf8(&password).map_err(|e| format!("Invalid UTF-8 in keychain: {e}"))?;

    let creds: KeychainCredentials =
        serde_json::from_str(json_str).map_err(|e| format!("Failed to parse keychain JSON: {e}"))?;

    let oauth = creds
        .claude_ai_oauth
        .ok_or_else(|| "No claudeAiOauth found in keychain credentials. Is Claude Code signed in?".to_string())?;

    Ok(Tokens {
        access_token: oauth.access_token,
        refresh_token: oauth.refresh_token,
    })
}

pub async fn refresh_access_token(
    client: &reqwest::Client,
    refresh_token: &str,
) -> Result<String, String> {
    let params = [
        ("grant_type", "refresh_token"),
        ("client_id", CLIENT_ID),
        ("refresh_token", refresh_token),
    ];

    let resp = client
        .post(TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Token refresh request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Token refresh failed ({status}): {body}"));
    }

    let token_resp: TokenResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse token response: {e}"))?;

    Ok(token_resp.access_token)
}
