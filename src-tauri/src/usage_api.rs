use serde::{Deserialize, Serialize};

const USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";

#[derive(Debug, Deserialize)]
struct ApiBucket {
    utilization: f64,
    resets_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    five_hour: Option<ApiBucket>,
    seven_day: Option<ApiBucket>,
    seven_day_opus: Option<ApiBucket>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageBucket {
    pub utilization: f64,
    pub resets_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageData {
    pub five_hour: Option<UsageBucket>,
    pub seven_day: Option<UsageBucket>,
    pub seven_day_opus: Option<UsageBucket>,
    pub fetched_at: String,
}

pub async fn fetch_usage(
    client: &reqwest::Client,
    access_token: &str,
) -> Result<UsageData, String> {
    let resp = client
        .get(USAGE_URL)
        .header("Authorization", format!("Bearer {access_token}"))
        .header("anthropic-beta", "oauth-2025-04-20")
        .send()
        .await
        .map_err(|e| format!("Usage request failed: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Usage API error ({status}): {body}"));
    }

    let api: ApiResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse usage response: {e}"))?;

    let now = chrono::Utc::now().to_rfc3339();

    Ok(UsageData {
        five_hour: api.five_hour.map(|b| UsageBucket {
            utilization: b.utilization,
            resets_at: b.resets_at,
        }),
        seven_day: api.seven_day.map(|b| UsageBucket {
            utilization: b.utilization,
            resets_at: b.resets_at,
        }),
        seven_day_opus: api.seven_day_opus.map(|b| UsageBucket {
            utilization: b.utilization,
            resets_at: b.resets_at,
        }),
        fetched_at: now,
    })
}
