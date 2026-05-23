use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct DnsCache {
    entries: HashMap<String, CacheEntry>,
    max_size: usize,
}

struct CacheEntry {
    ips: Vec<String>,
    expires_at: Instant,
}

impl DnsCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: HashMap::new(),
            max_size,
        }
    }

    pub fn get(&self, hostname: &str) -> Option<Vec<String>> {
        self.entries.get(hostname).and_then(|entry| {
            if entry.expires_at > Instant::now() {
                Some(entry.ips.clone())
            } else {
                None
            }
        })
    }

    pub fn set(&mut self, hostname: String, ips: Vec<String>, ttl_secs: u64) {
        // Evict oldest entries if at capacity
        let to_evict = self.max_size.saturating_sub(self.entries.len());

        if to_evict == 0 {
            // First try removing expired entries
            let expired_keys: Vec<String> = self.entries.iter()
                .filter(|(_, e)| e.expires_at <= Instant::now())
                .map(|(k, _)| k.clone())
                .collect();

            for key in expired_keys {
                self.entries.remove(&key);
            }

            // If still at capacity, remove oldest non-expired entries
            if self.entries.len() >= self.max_size {
                let mut by_expiry: Vec<(String, Instant)> = self.entries.iter()
                    .filter(|(_, e)| e.expires_at > Instant::now())
                    .map(|(k, e)| (k.clone(), e.expires_at))
                    .collect();

                by_expiry.sort_by_key(|(_, e)| *e);

                let to_remove = self.max_size - self.entries.len() + 1;
                for (key, _) in by_expiry.iter().take(to_remove) {
                    self.entries.remove(key);
                }
            }
        }

        self.entries.insert(
            hostname,
            CacheEntry {
                ips,
                expires_at: Instant::now() + Duration::from_secs(ttl_secs),
            },
        );
    }

    pub fn clear_expired(&mut self) {
        self.entries.retain(|_, e| e.expires_at > Instant::now());
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}
