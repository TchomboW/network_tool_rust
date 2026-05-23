use surge_ping::{Client, Config, PingIdentifier, PingSequence};
use std::net::IpAddr;
use std::time::Duration;

use crate::models::PingStats;

pub async fn ping(target: &str, count: u32) -> Result<PingStats, anyhow::Error> {
    let addr = target.parse::<IpAddr>()?;

    let config = Config::default();
    let client = Client::new(&config)?;
    let mut pinger = client.pinger(addr, PingIdentifier(0)).await;

    let mut min_latency: Option<f64> = None;
    let mut max_latency: Option<f64> = None;
    let mut total_latency: f64 = 0.0;
    let mut received = 0u32;

    for i in 0..count {
        let seq = PingSequence::from(i as u16);

        match tokio::time::timeout(Duration::from_secs(5), pinger.ping(seq, &[])).await {
            Ok(reply) => match reply {
                Ok((_packet, duration)) => {
                    received += 1;
                    let latency = duration.as_millis() as f64;
                    total_latency += latency;

                    if let Some(m) = min_latency {
                        if latency < m { min_latency = Some(latency); }
                    } else {
                        min_latency = Some(latency);
                    }

                    if let Some(m) = max_latency {
                        if latency > m { max_latency = Some(latency); }
                    } else {
                        max_latency = Some(latency);
                    }
                }
                Err(_) => { /* packet lost */ }
            },
            Err(_) => { /* timeout, treat as lost */ }
        }

        if i < count - 1 {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    let loss_percent = if count > 0 {
        ((count - received) as f64 / count as f64) * 100.0
    } else {
        0.0
    };

    Ok(PingStats {
        transmitted: count,
        received,
        loss_percent,
        min_latency_ms: min_latency,
        avg_latency_ms: if received > 0 { Some(total_latency / received as f64) } else { None },
        max_latency_ms: max_latency,
    })
}
