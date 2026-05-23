use std::time::Duration;

pub async fn retry_with_backoff<F, Fut, T>(
    max_retries: u32,
    base_delay: Duration,
    mut operation: F,
) -> Result<T, anyhow::Error>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, anyhow::Error>>,
{
    let mut delay = base_delay;

    for attempt in 0..=max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt < max_retries => {
                eprintln!(
                    "  Attempt {} failed: {}. Retrying in {:?}...",
                    attempt + 1, e, delay
                );
                tokio::time::sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
            Err(e) => {
                eprintln!("  All {} attempts failed: {}", max_retries + 1, e);
                return Err(e);
            }
        }
    }

    unreachable!()
}
