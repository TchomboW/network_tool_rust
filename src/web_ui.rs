use axum::{
    extract::Query,
    http::StatusCode,
    response::Json,
    routing::{get},
    Router,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct DiagnosticsQuery {
    #[serde(default = "default_target")]
    pub target: String,
    #[serde(default = "default_ping_count")]
    pub ping_count: u32,
    #[serde(default = "default_tcp_port")]
    pub tcp_port: u16,
}

fn default_target() -> String { "8.8.8.8".into() }
fn default_ping_count() -> u32 { 4 }
fn default_tcp_port() -> u16 { 443 }

#[derive(Serialize, Deserialize, Debug)]
pub struct DiagnosticsResponse {
    pub timestamp: String,
    pub target: String,
    pub ping: DiagnosticsCheck<PingData>,
    pub dns: DiagnosticsCheck<DnsData>,
    pub tcp: DiagnosticsCheck<TcpData>,
    pub http: DiagnosticsCheck<HttpData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DiagnosticsCheck<T> {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PingData {
    pub transmitted: u32,
    pub received: u32,
    pub loss_percent: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_latency_ms: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_latency_ms: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_latency_ms: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DnsData {
    pub hostname: String,
    pub resolve_time_ms: f64,
    pub ips: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TcpData {
    pub port: u16,
    pub success: bool,
    pub duration_ms: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HttpData {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<u16>,
    pub status_text: String,
    pub ttfb_ms: f64,
}

fn compute_ping_status(p: &crate::models::PingStats) -> String {
    if p.received > 0 && p.loss_percent == 0.0 {
        "healthy".to_string()
    } else if p.received > 0 {
        "degraded".to_string()
    } else {
        "unhealthy".to_string()
    }
}

fn compute_dns_status(d: &crate::models::DnsResult) -> String {
    if !d.ips.is_empty() { "healthy".to_string() } else { "unhealthy".to_string() }
}

fn compute_tcp_status(t: &crate::models::TcpResult) -> String {
    if t.success { "healthy".to_string() } else { "unhealthy".to_string() }
}

fn compute_http_status(h: &crate::models::HttpResult) -> String {
    if let Some(code) = h.status_code {
        if code >= 200 && code < 400 { "healthy".to_string() } else { "degraded".to_string() }
    } else {
        "unhealthy".to_string()
    }
}

pub async fn diagnostics_handler(
    Query(params): Query<DiagnosticsQuery>,
) -> Result<Json<DiagnosticsResponse>, StatusCode> {

    let target = &params.target;
    let timestamp = chrono::Utc::now().to_rfc3339();

    // Determine if target looks like a URL/hostname for HTTP check
    let http_target = if target.starts_with("http://") || target.starts_with("https://") {
        target.clone()
    } else {
        format!("https://{}", target)
    };

    // Run all diagnostics concurrently
    let ping_fut = crate::modules::icmp::ping(target, params.ping_count);
    let dns_fut = crate::modules::dns::resolve_dns(target);
    let tcp_fut = crate::modules::tcp::tcp_connect(target, params.tcp_port);
    let http_fut = crate::modules::http::check_http(&http_target);

    let (ping, dns, tcp, http) = tokio::join!(ping_fut, dns_fut, tcp_fut, http_fut);

    let ping_check = match ping {
        Ok(p) => DiagnosticsCheck {
            status: compute_ping_status(&p),
            error: None,
            data: Some(PingData {
                transmitted: p.transmitted, received: p.received, loss_percent: p.loss_percent,
                min_latency_ms: p.min_latency_ms, avg_latency_ms: p.avg_latency_ms, max_latency_ms: p.max_latency_ms,
            }),
        },
        Err(e) => DiagnosticsCheck {
            status: "error".into(), error: Some(e.to_string()), data: None,
        },
    };

    let dns_check = match dns {
        Ok(d) => DiagnosticsCheck {
            status: compute_dns_status(&d),
            error: None,
            data: Some(DnsData { hostname: d.hostname, resolve_time_ms: d.resolve_time_ms, ips: d.ips }),
        },
        Err(e) => DiagnosticsCheck {
            status: "error".into(), error: Some(e.to_string()), data: None,
        },
    };

    let tcp_check = match tcp {
        Ok(t) => DiagnosticsCheck {
            status: compute_tcp_status(&t),
            error: None,
            data: Some(TcpData { port: t.port, success: t.success, duration_ms: t.duration_ms }),
        },
        Err(e) => DiagnosticsCheck {
            status: "error".into(), error: Some(e.to_string()), data: None,
        },
    };

    let http_check = match http {
        Ok(h) => DiagnosticsCheck {
            status: compute_http_status(&h),
            error: None,
            data: Some(HttpData { url: h.url, status_code: h.status_code, status_text: h.status_text, ttfb_ms: h.ttfb_ms }),
        },
        Err(e) => DiagnosticsCheck {
            status: "error".into(), error: Some(e.to_string()), data: None,
        },
    };

    Ok(Json(DiagnosticsResponse {
        timestamp, target: params.target.clone(),
        ping: ping_check, dns: dns_check, tcp: tcp_check, http: http_check,
    }))
}

pub async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

pub fn create_router() -> Router {
    Router::new()
        .route("/api/health", get(health_handler))
        .route("/api/diagnostics", get(diagnostics_handler))
}
