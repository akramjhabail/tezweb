use std::time::Duration;
use tokio::time::timeout;
use std::future::Future;

pub struct RequestTimeout {
    pub duration: Duration,
}

impl RequestTimeout {
    pub fn new(secs: u64) -> Self {
        Self { duration: Duration::from_secs(secs) }
    }

    pub async fn wrap<F, T>(&self, fut: F) -> Option<T>
    where
        F: Future<Output = T>,
    {
        timeout(self.duration, fut).await.ok()
    }
}

/// Usage: request_timeout(30) → 30 second timeout
pub fn request_timeout(secs: u64) -> RequestTimeout {
    RequestTimeout::new(secs)
}