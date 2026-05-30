use std::time::Duration;

use crate::models::TcpResult;

pub async fn tcp_connect(target: &str, port: u16) -> Result<TcpResult, anyhow::Error> {
    let addr = format!("{}:{}", target, port);
    let start = std::time::Instant::now();

    // Add 10-second timeout to prevent hanging on unreachable hosts
    match tokio::time::timeout(Duration::from_secs(10), tokio::net::TcpStream::connect(&addr)).await {
        Ok(Ok(_stream)) => {
            let elapsed = start.elapsed().as_millis() as f64;
            Ok(TcpResult {
                port,
                success: true,
                duration_ms: elapsed,
            })
        }
        Ok(Err(_)) => {
            let elapsed = start.elapsed().as_millis() as f64;
            Ok(TcpResult {
                port,
                success: false,
                duration_ms: elapsed,
            })
        }
        Err(_) => {
            // Timeout occurred
            Ok(TcpResult {
                port,
                success: false,
                duration_ms: 10000.0,
            })
        }
    }
}
