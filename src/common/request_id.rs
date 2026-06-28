//! Request ID — Unique ID har request ke liye
//! Format: tez-{hex8}-{counter4}
//! Example: tez-a1b2c3d4-0001

use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(1);

/// Unique Request ID generate karo
pub fn generate_request_id() -> String {
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);

    // Fast pseudo-random — no external crate!
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();

    let hash = time ^ (count as u32).wrapping_mul(0x9e3779b9);

    format!("tez-{:08x}-{:04}", hash, count % 10000)
}