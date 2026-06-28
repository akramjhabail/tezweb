pub struct BodyLimit {
    pub max_bytes: usize,
}

impl BodyLimit {
    pub fn new(max_bytes: usize) -> Self {
        Self { max_bytes }
    }

    /// Ok(()) → size theek hai
    /// Err(String) → size zyada hai
    pub fn check(&self, body: &[u8]) -> Result<(), String> {
        if body.len() > self.max_bytes {
            return Err(format!(
                "Body too large: {} bytes exceeds limit of {} bytes",
                body.len(),
                self.max_bytes
            ));
        }
        Ok(())
    }
}

/// Usage: body_limit(10) → 10 MB limit
pub fn body_limit(max_mb: usize) -> BodyLimit {
    BodyLimit::new(max_mb * 1024 * 1024)
}