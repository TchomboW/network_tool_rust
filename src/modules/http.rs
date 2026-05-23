use reqwest::Client;

use crate::models::HttpResult;

pub async fn check_http(target: &str) -> Result<HttpResult, anyhow::Error> {
    let url = if target.starts_with("http://") || target.starts_with("https://") {
        target.to_string()
    } else {
        format!("https://{}", target)
    };

    let client = Client::builder().timeout(std::time::Duration::from_secs(10)).build()?;
    let start = std::time::Instant::now();

    match client.get(&url).send().await {
        Ok(response) => {
            let elapsed = start.elapsed().as_millis() as f64;
            let status_code = response.status().as_u16();
            let status_text = format!(
                "{} {}",
                status_code,
                reqwest::StatusCode::from_u16(status_code)
                    .map(|s| s.canonical_reason().unwrap_or("Unknown").to_string())
                    .unwrap_or_else(|_| "Unknown".to_string()),
            );

            Ok(HttpResult {
                url,
                status_code: Some(status_code),
                status_text,
                ttfb_ms: elapsed,
            })
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as f64;
            Ok(HttpResult {
                url,
                status_code: None,
                status_text: format!("Error: {}", e),
                ttfb_ms: elapsed,
            })
        }
    }
}
