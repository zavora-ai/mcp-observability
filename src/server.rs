use crate::client::ObsBackend;
use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde_json::json;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct EmptyInput {}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct IdInput { pub id: String }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct LogQueryInput { pub query: String, pub start: Option<String>, pub end: Option<String>, pub limit: Option<u32>, pub service: Option<String>, pub level: Option<String> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MetricQueryInput { pub metric: String, pub start: Option<String>, pub end: Option<String>, pub granularity: Option<String>, pub tags: Option<serde_json::Value> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TraceQueryInput { pub service: Option<String>, pub operation: Option<String>, pub min_duration_ms: Option<u64>, pub status: Option<String>, pub limit: Option<u32> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AlertFilterInput { pub status: Option<String>, pub severity: Option<String>, pub service: Option<String> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CreateAlertInput { pub name: String, pub metric: String, pub condition: String, pub threshold: f64, pub severity: String, pub notify: Option<Vec<String>> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AckAlertInput { pub alert_id: String, pub message: Option<String> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct IncidentInput { pub title: String, pub severity: String, pub service: String, pub description: Option<String> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct UpdateIncidentInput { pub id: String, pub status: Option<String>, pub resolution: Option<String> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SloInput { pub service: Option<String> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ServiceInput { pub service: String }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RunbookInput { pub service: Option<String>, pub alert_type: Option<String> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ResolveAlertInput { pub alert_id: String, pub resolution: Option<String> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AlertHistoryInput { pub alert_id: Option<String>, pub service: Option<String>, pub limit: Option<u32> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MonitorCreateInput { pub name: String, pub url: String, pub monitor_type: Option<String>, pub interval_seconds: Option<u32> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MonitorCheckInput { pub url: String, pub timeout_seconds: Option<u64> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PromQueryInput { pub query: String, pub start: Option<String>, pub end: Option<String>, pub step: Option<String> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GrafanaSyncInput { pub direction: String, pub dashboard_id: Option<String> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct IncidentResolveInput { pub id: String, pub root_cause: String, pub resolution: String, pub follow_up: Option<Vec<String>> }

#[derive(Clone)]
pub struct ObsServer { pub backend: ObsBackend }

fn r(result: Result<serde_json::Value, anyhow::Error>) -> String {
    match result { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {}", e) }
}

#[tool_router]
impl ObsServer {
    // === Logs (4) ===

    #[tool(description = "Search logs by query, time range, service, and level (error/warn/info/debug)")]
    async fn query_logs(&self, Parameters(input): Parameters<LogQueryInput>) -> String {
        let mut path = format!("/logs/search?query={}", urlencoding::encode(&input.query));
        if let Some(s) = &input.start { path.push_str(&format!("&start={}", s)); }
        if let Some(e) = &input.end { path.push_str(&format!("&end={}", e)); }
        if let Some(l) = input.limit { path.push_str(&format!("&limit={}", l)); }
        if let Some(s) = &input.service { path.push_str(&format!("&service={}", s)); }
        if let Some(l) = &input.level { path.push_str(&format!("&level={}", l)); }
        r(self.backend.api.get(&path).await)
    }

    #[tool(description = "Get log volume and error rate over time for a service")]
    async fn get_log_stats(&self, Parameters(input): Parameters<ServiceInput>) -> String {
        r(self.backend.api.get(&format!("/logs/stats?service={}", input.service)).await)
    }

    #[tool(description = "Get recent error logs with stack traces")]
    async fn get_errors(&self, Parameters(input): Parameters<ServiceInput>) -> String {
        r(self.backend.api.get(&format!("/logs/search?level=error&service={}&limit=20", input.service)).await)
    }

    #[tool(description = "Tail live logs for a service (last N entries)")]
    async fn tail_logs(&self, Parameters(input): Parameters<ServiceInput>) -> String {
        r(self.backend.api.get(&format!("/logs/tail?service={}&limit=50", input.service)).await)
    }

    // === Metrics (4) ===

    #[tool(description = "Query a metric over time (CPU, memory, latency, error_rate, throughput)")]
    async fn query_metric(&self, Parameters(input): Parameters<MetricQueryInput>) -> String {
        let mut path = format!("/metrics/query?metric={}", urlencoding::encode(&input.metric));
        if let Some(s) = &input.start { path.push_str(&format!("&start={}", s)); }
        if let Some(e) = &input.end { path.push_str(&format!("&end={}", e)); }
        if let Some(g) = &input.granularity { path.push_str(&format!("&granularity={}", g)); }
        r(self.backend.api.get(&path).await)
    }

    #[tool(description = "List available metrics for a service")]
    async fn list_metrics(&self, Parameters(input): Parameters<ServiceInput>) -> String {
        r(self.backend.api.get(&format!("/metrics/list?service={}", input.service)).await)
    }

    #[tool(description = "Get current system health: CPU, memory, disk, network across services")]
    async fn get_system_health(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        r(self.backend.api.get("/metrics/health").await)
    }

    #[tool(description = "Compare a metric across services or time periods")]
    async fn compare_metrics(&self, Parameters(input): Parameters<MetricQueryInput>) -> String {
        let mut path = format!("/metrics/compare?metric={}", urlencoding::encode(&input.metric));
        if let Some(t) = &input.tags { path.push_str(&format!("&tags={}", urlencoding::encode(&t.to_string()))); }
        r(self.backend.api.get(&path).await)
    }

    // === Traces (4) ===

    #[tool(description = "Search distributed traces by service, operation, duration, or status")]
    async fn search_traces(&self, Parameters(input): Parameters<TraceQueryInput>) -> String {
        let mut path = "/traces/search?".to_string();
        if let Some(s) = &input.service { path.push_str(&format!("service={}&", s)); }
        if let Some(o) = &input.operation { path.push_str(&format!("operation={}&", o)); }
        if let Some(d) = input.min_duration_ms { path.push_str(&format!("min_duration_ms={}&", d)); }
        if let Some(s) = &input.status { path.push_str(&format!("status={}&", s)); }
        if let Some(l) = input.limit { path.push_str(&format!("limit={}&", l)); }
        r(self.backend.api.get(&path).await)
    }

    #[tool(description = "Get a full trace with all spans, timings, and errors")]
    async fn get_trace(&self, Parameters(input): Parameters<IdInput>) -> String {
        r(self.backend.api.get(&format!("/traces/{}", input.id)).await)
    }

    #[tool(description = "Get service map showing dependencies and latencies")]
    async fn get_service_map(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        r(self.backend.api.get("/traces/service-map").await)
    }

    #[tool(description = "Find slow endpoints: p50, p95, p99 latencies by operation")]
    async fn get_latency_breakdown(&self, Parameters(input): Parameters<ServiceInput>) -> String {
        r(self.backend.api.get(&format!("/traces/latency?service={}", input.service)).await)
    }

    // === Alerts (4) ===

    #[tool(description = "List active alerts filtered by status (firing/resolved), severity, service")]
    async fn list_alerts(&self, Parameters(input): Parameters<AlertFilterInput>) -> String {
        let mut path = "/alerts?".to_string();
        if let Some(s) = &input.status { path.push_str(&format!("status={}&", s)); }
        if let Some(sev) = &input.severity { path.push_str(&format!("severity={}&", sev)); }
        if let Some(svc) = &input.service { path.push_str(&format!("service={}&", svc)); }
        r(self.backend.api.get(&path).await)
    }

    #[tool(description = "Get alert details with history and related metrics")]
    async fn get_alert(&self, Parameters(input): Parameters<IdInput>) -> String {
        r(self.backend.api.get(&format!("/alerts/{}", input.id)).await)
    }

    #[tool(description = "Create a new alert rule (metric threshold, anomaly detection)")]
    async fn create_alert(&self, Parameters(input): Parameters<CreateAlertInput>) -> String {
        r(self.backend.api.post("/alerts", &json!({
            "name": input.name, "metric": input.metric, "condition": input.condition,
            "threshold": input.threshold, "severity": input.severity, "notify": input.notify
        })).await)
    }

    #[tool(description = "Acknowledge a firing alert with optional message")]
    async fn acknowledge_alert(&self, Parameters(input): Parameters<AckAlertInput>) -> String {
        r(self.backend.api.post(&format!("/alerts/{}/acknowledge", input.alert_id), &json!({"message": input.message})).await)
    }

    // === Incidents (4) ===

    #[tool(description = "List incidents (open, investigating, resolved)")]
    async fn list_incidents(&self, Parameters(input): Parameters<AlertFilterInput>) -> String {
        let mut path = "/incidents?".to_string();
        if let Some(s) = &input.status { path.push_str(&format!("status={}&", s)); }
        if let Some(sev) = &input.severity { path.push_str(&format!("severity={}&", sev)); }
        r(self.backend.api.get(&path).await)
    }

    #[tool(description = "Get incident details: timeline, affected services, responders")]
    async fn get_incident(&self, Parameters(input): Parameters<IdInput>) -> String {
        r(self.backend.api.get(&format!("/incidents/{}", input.id)).await)
    }

    #[tool(description = "Create a new incident")]
    async fn create_incident(&self, Parameters(input): Parameters<IncidentInput>) -> String {
        r(self.backend.api.post("/incidents", &json!({
            "title": input.title, "severity": input.severity,
            "service": input.service, "description": input.description
        })).await)
    }

    #[tool(description = "Update incident status or add resolution notes")]
    async fn update_incident(&self, Parameters(input): Parameters<UpdateIncidentInput>) -> String {
        let mut body = json!({});
        if let Some(s) = input.status { body["status"] = json!(s); }
        if let Some(r) = input.resolution { body["resolution"] = json!(r); }
        r(self.backend.api.patch(&format!("/incidents/{}", input.id), &body).await)
    }

    // === SLOs (3) ===

    #[tool(description = "List SLOs with current burn rate and error budget remaining")]
    async fn list_slos(&self, Parameters(input): Parameters<SloInput>) -> String {
        let mut path = "/slos?".to_string();
        if let Some(s) = &input.service { path.push_str(&format!("service={}&", s)); }
        r(self.backend.api.get(&path).await)
    }

    #[tool(description = "Get SLO details: target, current value, burn rate, budget")]
    async fn get_slo(&self, Parameters(input): Parameters<IdInput>) -> String {
        r(self.backend.api.get(&format!("/slos/{}", input.id)).await)
    }

    #[tool(description = "Get SLO breach forecast — when will error budget run out?")]
    async fn forecast_slo(&self, Parameters(input): Parameters<IdInput>) -> String {
        r(self.backend.api.get(&format!("/slos/{}/forecast", input.id)).await)
    }

    // === Dashboards & Runbooks (3) ===

    #[tool(description = "List observability dashboards")]
    async fn list_dashboards(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        r(self.backend.api.get("/dashboards").await)
    }

    #[tool(description = "Get dashboard with all panels and current values")]
    async fn get_dashboard(&self, Parameters(input): Parameters<IdInput>) -> String {
        r(self.backend.api.get(&format!("/dashboards/{}", input.id)).await)
    }

    #[tool(description = "Find relevant runbook for an alert or service issue")]
    async fn get_runbook(&self, Parameters(input): Parameters<RunbookInput>) -> String {
        let mut path = "/runbooks?".to_string();
        if let Some(s) = &input.service { path.push_str(&format!("service={}&", s)); }
        if let Some(a) = &input.alert_type { path.push_str(&format!("alert_type={}&", a)); }
        r(self.backend.api.get(&path).await)
    }

    // === Services (2) ===

    #[tool(description = "List all monitored services with health status")]
    async fn list_services(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        r(self.backend.api.get("/services").await)
    }

    #[tool(description = "Get service overview: health, dependencies, recent alerts, SLOs")]
    async fn get_service(&self, Parameters(input): Parameters<ServiceInput>) -> String {
        r(self.backend.api.get(&format!("/services/{}", input.service)).await)
    }

    // === New: Alert Resolve & History ===

    #[tool(description = "Resolve a firing alert (marks as resolved, stops notifications)")]
    async fn resolve_alert(&self, Parameters(input): Parameters<ResolveAlertInput>) -> String {
        let body = json!({"status": "resolved", "resolution": input.resolution});
        r(self.backend.api.post(&format!("/alerts/{}/resolve", input.alert_id), &body).await)
    }

    #[tool(description = "Get alert firing history (past incidents of an alert rule firing)")]
    async fn alert_history(&self, Parameters(input): Parameters<AlertHistoryInput>) -> String {
        let mut params = vec![];
        if let Some(id) = &input.alert_id { params.push(format!("alert_id={}", id)); }
        if let Some(svc) = &input.service { params.push(format!("service={}", svc)); }
        params.push(format!("limit={}", input.limit.unwrap_or(20)));
        r(self.backend.api.get(&format!("/alerts/history?{}", params.join("&"))).await)
    }

    // === New: Uptime Monitors ===

    #[tool(description = "Create an uptime monitor (HTTP, TCP, or ping check at regular intervals)")]
    async fn monitor_create(&self, Parameters(input): Parameters<MonitorCreateInput>) -> String {
        let body = json!({"name": input.name, "url": input.url, "type": input.monitor_type.unwrap_or_else(|| "http".into()), "interval_seconds": input.interval_seconds.unwrap_or(60)});
        r(self.backend.api.post("/monitors", &body).await)
    }

    #[tool(description = "Check uptime status of a URL right now (performs live HTTP request and returns status, latency)")]
    async fn monitor_status(&self, Parameters(input): Parameters<MonitorCheckInput>) -> String {
        let timeout = input.timeout_seconds.unwrap_or(10);
        let start = std::time::Instant::now();
        let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(timeout)).build().unwrap_or_default();
        match client.get(&input.url).send().await {
            Ok(resp) => {
                let latency_ms = start.elapsed().as_millis();
                let status_code = resp.status().as_u16();
                let up = status_code < 400;
                json!({"url": input.url, "status": if up { "up" } else { "degraded" }, "status_code": status_code, "latency_ms": latency_ms}).to_string()
            }
            Err(e) => json!({"url": input.url, "status": "down", "error": e.to_string()}).to_string(),
        }
    }

    // === New: Backend Sync ===

    #[tool(description = "Query Prometheus directly using PromQL. Requires PROMETHEUS_URL env var.")]
    async fn sync_prometheus(&self, Parameters(input): Parameters<PromQueryInput>) -> String {
        let prom_url = std::env::var("PROMETHEUS_URL").unwrap_or_else(|_| "http://localhost:9090".into());
        let mut url = format!("{}/api/v1/query?query={}", prom_url, input.query);
        if let Some(ref start) = input.start { url = format!("{}/api/v1/query_range?query={}&start={}&end={}&step={}", prom_url, input.query, start, input.end.as_deref().unwrap_or("now"), input.step.as_deref().unwrap_or("60s")); }
        let client = reqwest::Client::new();
        match client.get(&url).send().await {
            Ok(resp) => match resp.json::<serde_json::Value>().await {
                Ok(data) => json!({"source": "prometheus", "status": data["status"], "result_type": data["data"]["resultType"], "results": data["data"]["result"]}).to_string(),
                Err(e) => json!({"error": e.to_string()}).to_string(),
            },
            Err(e) => json!({"error": "PROMETHEUS_UNAVAILABLE", "details": e.to_string(), "url": prom_url}).to_string(),
        }
    }

    #[tool(description = "Sync dashboards with Grafana. Pull imports dashboards, push exports. Requires GRAFANA_URL, GRAFANA_TOKEN env vars.")]
    async fn sync_grafana(&self, Parameters(input): Parameters<GrafanaSyncInput>) -> String {
        let grafana_url = std::env::var("GRAFANA_URL").unwrap_or_else(|_| "http://localhost:3000".into());
        let token = match std::env::var("GRAFANA_TOKEN") {
            Ok(t) => t,
            Err(_) => return json!({"error": "NOT_CONFIGURED", "message": "Set GRAFANA_URL and GRAFANA_TOKEN"}).to_string(),
        };
        let client = reqwest::Client::new();
        match input.direction.as_str() {
            "pull" => {
                let url = format!("{}/api/search?type=dash-db", grafana_url);
                match client.get(&url).header("Authorization", format!("Bearer {}", token)).send().await {
                    Ok(resp) => match resp.json::<serde_json::Value>().await {
                        Ok(data) => json!({"source": "grafana", "status": "pulled", "dashboards": data}).to_string(),
                        Err(e) => json!({"error": e.to_string()}).to_string(),
                    },
                    Err(e) => json!({"error": e.to_string()}).to_string(),
                }
            }
            _ => json!({"status": "push_supported", "message": "Use Grafana API POST /api/dashboards/db"}).to_string(),
        }
    }

    // === New: Incident Resolve with RCA ===

    #[tool(description = "Resolve an incident with root cause analysis, resolution details, and follow-up actions")]
    async fn incident_resolve(&self, Parameters(input): Parameters<IncidentResolveInput>) -> String {
        let body = json!({"status": "resolved", "root_cause": input.root_cause, "resolution": input.resolution, "follow_up_actions": input.follow_up});
        r(self.backend.api.post(&format!("/incidents/{}/resolve", input.id), &body).await)
    }
}

adk_mcp_sdk::mcp_2026_server! {
    server: ObsServer,
    task_tools: ["forecast_slo", "sync_prometheus", "sync_grafana"],
    approval_tools: [],
    cache_ttl_ms: 60_000,
}
