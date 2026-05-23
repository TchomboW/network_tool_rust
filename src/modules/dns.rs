use hickory_resolver::TokioAsyncResolver;

use crate::models::DnsResult;

pub async fn resolve_dns(hostname: &str) -> Result<DnsResult, anyhow::Error> {
    let resolver = TokioAsyncResolver::tokio_from_system_conf()?;

    let start = std::time::Instant::now();
    let ips: Vec<String> = match resolver.lookup_ip(hostname).await {
        Ok(lookup) => lookup.iter().map(|ip| ip.to_string()).collect(),
        Err(_) => Vec::new(),
    };

    let elapsed = start.elapsed().as_millis() as f64;

    Ok(DnsResult {
        hostname: hostname.to_string(),
        resolve_time_ms: elapsed,
        ips,
    })
}
