# Observability MCP Server

[![Crates.io](https://img.shields.io/crates/v/mcp-observability.svg)](https://crates.io/crates/mcp-observability)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![ADK-Rust Enterprise](https://img.shields.io/badge/ADK--Rust-Enterprise-purple.svg)](https://enterprise.adk-rust.com)
[![Registry Ready](https://img.shields.io/badge/ADK_Registry-Ready-green.svg)](https://www.zavora.ai)

Full-stack observability for AI agents — logs, metrics, distributed traces, alerts, incidents, SLOs, dashboards, service maps, and runbooks. 28 tools for debugging, monitoring, and incident response.

## Architecture

<p align="center">
  <img src="https://raw.githubusercontent.com/zavora-ai/mcp-observability/main/docs/assets/architecture.svg" alt="MCP Observability Architecture" width="850"/>
</p>

## Tools (28)

### Logs (4)

| Tool | Purpose |
|------|---------|
| `query_logs` | Search logs by query, time, service, level |
| `get_log_stats` | Log volume and error rate over time |
| `get_errors` | Recent errors with stack traces |
| `tail_logs` | Live tail (last 50 entries) |

### Metrics (4)

| Tool | Purpose |
|------|---------|
| `query_metric` | Query metric time-series (CPU, latency, etc.) |
| `list_metrics` | Available metrics for a service |
| `get_system_health` | Current CPU/memory/disk across services |
| `compare_metrics` | Compare metric across services or periods |

### Traces (4)

| Tool | Purpose |
|------|---------|
| `search_traces` | Find traces by service, duration, status |
| `get_trace` | Full trace with all spans and timings |
| `get_service_map` | Service dependency graph with latencies |
| `get_latency_breakdown` | p50/p95/p99 by operation |

### Alerts (4)

| Tool | Purpose |
|------|---------|
| `list_alerts` | Active alerts (filter: status, severity, service) |
| `get_alert` | Alert details + history + related metrics |
| `create_alert` | Create alert rule (threshold/anomaly) |
| `acknowledge_alert` | Ack a firing alert |

### Incidents (4)

| Tool | Purpose |
|------|---------|
| `list_incidents` | Open/investigating/resolved incidents |
| `get_incident` | Timeline, affected services, responders |
| `create_incident` | Declare a new incident |
| `update_incident` | Update status or add resolution |

### SLOs (3)

| Tool | Purpose |
|------|---------|
| `list_slos` | SLOs with burn rate and error budget |
| `get_slo` | SLO target vs current value |
| `forecast_slo` | When will error budget run out? |

### Dashboards & Runbooks (3)

| Tool | Purpose |
|------|---------|
| `list_dashboards` | Available dashboards |
| `get_dashboard` | Dashboard with panels and values |
| `get_runbook` | Find runbook for alert/service issue |

### Services (2)

| Tool | Purpose |
|------|---------|
| `list_services` | All monitored services + health |
| `get_service` | Service overview: health, deps, alerts, SLOs |

## Installation

```bash
cargo install mcp-observability
```

## Configuration

| Backend | Env Vars | Provides |
|---------|----------|----------|
| **Datadog** | `DATADOG_API_KEY` + `DATADOG_APP_KEY` | Logs, metrics, traces, monitors, dashboards |
| **Grafana Cloud** | `GRAFANA_URL` + `GRAFANA_API_TOKEN` | Loki (logs), Prometheus (metrics), Tempo (traces) |
| **New Relic** | `NEWRELIC_API_KEY` + `NEWRELIC_ACCOUNT_ID` | APM, logs, dashboards, alerts |
| **Custom API** | `OBSERVABILITY_API_URL` + `OBSERVABILITY_API_KEY` | Your own monitoring stack |

## Client Configuration

```json
{
  "mcpServers": {
    "observability": {
      "command": "mcp-observability",
      "args": [],
      "env": {
        "DATADOG_API_KEY": "your-api-key",
        "DATADOG_APP_KEY": "your-app-key"
      }
    }
  }
}
```

## Usage Examples

### Debug a production issue
```
"Why is the API slow?"
→ get_system_health() — CPU normal, memory normal
→ get_latency_breakdown(service="api-gateway") — p99 jumped from 200ms to 2s
→ search_traces(service="api-gateway", min_duration_ms=1000) — find slow traces
→ get_trace(id="trace-abc") — database span taking 1.8s
→ query_logs(query="slow query", service="postgres") — found the culprit
```

### Incident response
```
"There's a spike in errors"
→ list_alerts(status="firing") — "Error rate > 5% on payment-service"
→ get_errors(service="payment-service") — NullPointerException in checkout
→ create_incident(title="Payment failures", severity="high", service="payment-service")
→ get_runbook(service="payment-service", alert_type="error_rate")
→ acknowledge_alert(alert_id="alert-123", message="Investigating")
```

### SLO monitoring
```
"Are we meeting our SLOs?"
→ list_slos() — API availability at 99.92% (target 99.9%) ✅, Latency SLO burning fast ⚠️
→ forecast_slo(id="slo-latency") — "Error budget exhausted in 3 days at current rate"
```

## MCP Server Manifest

```toml
server_id = "mcp_observability"
display_name = "Observability"
version = "1.0.0"
domain = "infrastructure"
risk_level = "low"
writes_allowed = "gated"
```

## License

Apache-2.0

---

Part of the [ADK-Rust Enterprise](https://enterprise.adk-rust.com) MCP server ecosystem.

Built with ❤️ by [Zavora AI](https://zavora.ai)

## rmcp and MCP compatibility

This server is built with [`rmcp` 3.1.2](https://github.com/modelcontextprotocol/rust-sdk/releases/tag/rmcp-v3.1.2) and requires Rust 1.88 or newer. The rmcp 3 rollout retains legacy MCP initialization compatibility and targets MCP protocol revisions `2025-11-25` and `2026-07-28`.
