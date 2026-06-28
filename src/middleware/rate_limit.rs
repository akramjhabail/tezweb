use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

struct RateEntry {
    count: AtomicU64,
    window_end: Instant,
}

pub struct RateLimiter {
    store: DashMap<String, RateEntry>,
    max_requests: u64,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: u64, window_secs: u64) -> Self {
        Self {
            store: DashMap::new(),
            max_requests,
            window: Duration::from_secs(window_secs),
        }
    }

    /// Ok(remaining) -> request allow
    /// Err -> limit exceed ho gayi
    pub fn check(&self, ip: &str) -> Result<u64, &'static str> {
        let now = Instant::now();
        let mut entry = self.store.entry(ip.to_string()).or_insert_with(|| RateEntry {
            count: AtomicU64::new(0),
            window_end: now + self.window,
        });

        // Window reset?
        if now >= entry.window_end {
            entry.count.store(0, Ordering::Relaxed);
            entry.window_end = now + self.window;
        }

        let count = entry.count.fetch_add(1, Ordering::Relaxed) + 1;

        if count > self.max_requests {
            return Err("Rate limit exceeded");
        }

        Ok(self.max_requests - count)
    }
}

/// 100 requests per 60 seconds per IP
/// Usage: rate_limit(100, 60)
pub fn rate_limit(max_requests: u64, window_secs: u64) -> RateLimiter {
    RateLimiter::new(max_requests, window_secs)
}
