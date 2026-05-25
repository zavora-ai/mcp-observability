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

#[derive(Clone)]
pub struct ObsServer { pub backend: ObsBackend }

fn r(result: Result<serde_json::Value, anyhow::Error>) -> String {
    match result { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {}", e) }
}

#[tool_router(server_handler)]
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
}
