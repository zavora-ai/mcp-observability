use anyhow::{Result, bail};
use reqwest::Client;
use serde_json::Value;

#[derive(Clone)]
pub struct ApiClient {
    pub http: Client,
    pub base_url: String,
    pub auth_header: String,
}

impl ApiClient {
    pub async fn get(&self, path: &str) -> Result<Value> {
        let resp = self.http.get(format!("{}{}", self.base_url, path))
            .header("Authorization", &self.auth_header)
            .send().await?;
        if !resp.status().is_success() { bail!("API {}: {}", resp.status(), resp.text().await?); }
        Ok(resp.json().await?)
    }

    pub async fn post(&self, path: &str, body: &Value) -> Result<Value> {
        let resp = self.http.post(format!("{}{}", self.base_url, path))
            .header("Authorization", &self.auth_header)
            .json(body).send().await?;
        if !resp.status().is_success() { bail!("API {}: {}", resp.status(), resp.text().await?); }
        Ok(resp.json().await?)
    }

    pub async fn patch(&self, path: &str, body: &Value) -> Result<Value> {
        let resp = self.http.patch(format!("{}{}", self.base_url, path))
            .header("Authorization", &self.auth_header)
            .json(body).send().await?;
        if !resp.status().is_success() { bail!("API {}: {}", resp.status(), resp.text().await?); }
        Ok(resp.json().await?)
    }
}

/// Observability backend — Datadog, Grafana, CloudWatch, or Custom API
#[derive(Clone)]
pub struct ObsBackend {
    pub api: ApiClient,
    pub provider: String,
}

impl ObsBackend {
    pub fn from_env() -> Result<Self> {
        // Datadog
        if let (Ok(api_key), Ok(app_key)) = (std::env::var("DATADOG_API_KEY"), std::env::var("DATADOG_APP_KEY")) {
            let site = std::env::var("DATADOG_SITE").unwrap_or("datadoghq.com".into());
            tracing::info!("Observability backend: Datadog ({})", site);
            return Ok(Self {
                api: ApiClient { http: Client::new(), base_url: format!("https://api.{}", site), auth_header: format!("DD-API-KEY: {}", api_key) },
                provider: "datadog".into(),
            });
            // Note: Datadog uses DD-API-KEY and DD-APPLICATION-KEY headers
            // We'll handle this specially in the server
        }
        // Grafana Cloud
        if let (Ok(url), Ok(token)) = (std::env::var("GRAFANA_URL"), std::env::var("GRAFANA_API_TOKEN")) {
            tracing::info!("Observability backend: Grafana");
            return Ok(Self {
                api: ApiClient { http: Client::new(), base_url: url.trim_end_matches('/').to_string(), auth_header: format!("Bearer {}", token) },
                provider: "grafana".into(),
            });
        }
        // New Relic
        if let Ok(api_key) = std::env::var("NEWRELIC_API_KEY") {
            let account_id = std::env::var("NEWRELIC_ACCOUNT_ID").unwrap_or_default();
            tracing::info!("Observability backend: New Relic");
            return Ok(Self {
                api: ApiClient { http: Client::new(), base_url: format!("https://api.newrelic.com/v2/accounts/{}", account_id), auth_header: format!("Api-Key {}", api_key) },
                provider: "newrelic".into(),
            });
        }
        // Custom API
        if let Ok(url) = std::env::var("OBSERVABILITY_API_URL") {
            let key = std::env::var("OBSERVABILITY_API_KEY").unwrap_or_default();
            tracing::info!("Observability backend: Custom API");
            return Ok(Self {
                api: ApiClient { http: Client::new(), base_url: url.trim_end_matches('/').to_string(), auth_header: format!("Bearer {}", key) },
                provider: "custom".into(),
            });
        }
        bail!("No observability backend. Set DATADOG_API_KEY+DATADOG_APP_KEY, GRAFANA_URL+GRAFANA_API_TOKEN, NEWRELIC_API_KEY, or OBSERVABILITY_API_URL")
    }
}
