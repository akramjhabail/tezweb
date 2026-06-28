//! Health Check — Auto endpoint for TezWeb

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::Instant;

// Server start time
static START_TIME: OnceLock<Instant> = OnceLock::new();

// Total requests counter
static REQUEST_COUNT: AtomicU64 = AtomicU64::new(0);

/// Server start hone par call karo
pub fn init_health() {
    START_TIME.get_or_init(Instant::now);
}

/// Har request par call karo
pub fn increment_requests() {
    REQUEST_COUNT.fetch_add(1, Ordering::Relaxed);
}

/// Uptime seconds mein
pub fn uptime_secs() -> u64 {
    START_TIME
        .get()
        .map(|t| t.elapsed().as_secs())
        .unwrap_or(0)
}

/// Total requests
pub fn total_requests() -> u64 {
    REQUEST_COUNT.load(Ordering::Relaxed)
}

/// Health JSON response
pub fn health_json() -> String {
    format!(
        r#"{{"status":"ok","uptime_secs":{},"requests_total":{},"version":"{}"}}"#,
        uptime_secs(),
        total_requests(),
        env!("CARGO_PKG_VERSION"),
    )
}// test
