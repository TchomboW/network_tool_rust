use std::time::Duration;

#[derive(Debug, Clone)]
pub struct DiagnosticResult {
    pub timestamp: String,
    pub run_number: u32,
    pub ping: Option<PingStats>,
    pub dns: Option<DnsResult>,
    pub tcp: Option<TcpResult>,
    pub http: Option<HttpResult>,
}

#[derive(Debug, Clone)]
pub struct PingStats {
    pub transmitted: u32,
    pub received: u32,
    pub loss_percent: f64,
    pub min_latency_ms: Option<f64>,
    pub avg_latency_ms: Option<f64>,
    pub max_latency_ms: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct DnsResult {
    pub hostname: String,
    pub resolve_time_ms: f64,
    pub ips: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TcpResult {
    pub port: u16,
    pub success: bool,
    pub duration_ms: f64,
}

#[derive(Debug, Clone)]
pub struct HttpResult {
    pub url: String,
    pub status_code: Option<u16>,
    pub status_text: String,
    pub ttfb_ms: f64,
}

#[derive(Debug, Clone)]
pub struct SpeedResult {
    pub download_mbps: f64,
    pub upload_mbps: f64,
}
