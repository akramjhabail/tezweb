pub struct CacheControl {
    pub max_age: Option<u64>,
    pub no_cache: bool,
    pub no_store: bool,
    pub public: bool,
    pub private: bool,
    pub immutable: bool,
}

impl CacheControl {
    pub fn new() -> Self {
        Self {
            max_age: None,
            no_cache: false,
            no_store: false,
            public: false,
            private: false,
            immutable: false,
        }
    }

    pub fn max_age(mut self, secs: u64) -> Self { self.max_age = Some(secs); self }
    pub fn no_cache(mut self) -> Self { self.no_cache = true; self }
    pub fn no_store(mut self) -> Self { self.no_store = true; self }
    pub fn public(mut self) -> Self   { self.public = true; self }
    pub fn private(mut self) -> Self  { self.private = true; self }
    pub fn immutable(mut self) -> Self { self.immutable = true; self }

    /// Header string banao
    pub fn to_header(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if self.no_store  { parts.push("no-store".to_string()); }
        if self.no_cache  { parts.push("no-cache".to_string()); }
        if self.public    { parts.push("public".to_string()); }
        if self.private   { parts.push("private".to_string()); }
        if self.immutable { parts.push("immutable".to_string()); }
        if let Some(age) = self.max_age {
            parts.push(format!("max-age={}", age));
        }
        parts.join(", ")
    }
}

impl Default for CacheControl {
    fn default() -> Self { Self::new() }
}

/// Usage examples:
/// CacheControl::new().public().max_age(3600)  → "public, max-age=3600"
/// CacheControl::new().no_store()               → "no-store"
/// CacheControl::new().private().max_age(0)     → "private, max-age=0"
pub fn cache_control() -> CacheControl {
    CacheControl::new()
}