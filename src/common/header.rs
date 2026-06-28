//! HTTP headers

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HeaderName {
    ContentType,
    ContentLength,
    Authorization,
    Accept,
    CacheControl,
    Connection,
    KeepAlive,
    Host,
    UserAgent,
    Custom(String),
}

impl HeaderName {
    pub fn as_str(&self) -> &str {
        match self {
            Self::ContentType    => "content-type",
            Self::ContentLength  => "content-length",
            Self::Authorization  => "authorization",
            Self::Accept         => "accept",
            Self::CacheControl   => "cache-control",
            Self::Connection     => "connection",
            Self::KeepAlive      => "keep-alive",
            Self::Host           => "host",
            Self::UserAgent      => "user-agent",
            Self::Custom(s)      => s,
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "content-type"   => Self::ContentType,
            "content-length" => Self::ContentLength,
            "authorization"  => Self::Authorization,
            "accept"         => Self::Accept,
            "cache-control"  => Self::CacheControl,
            "connection"     => Self::Connection,
            "keep-alive"     => Self::KeepAlive,
            "host"           => Self::Host,
            "user-agent"     => Self::UserAgent,
            _                => Self::Custom(s.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderValue {
    inner: String,
}

impl HeaderValue {
    pub fn new(value: impl Into<String>) -> Self {
        Self { inner: value.into() }
    }

    pub fn as_str(&self) -> &str {
        &self.inner
    }
}

impl From<&str> for HeaderValue {
    fn from(s: &str) -> Self { Self::new(s) }
}

impl From<String> for HeaderValue {
    fn from(s: String) -> Self { Self::new(s) }
}

#[derive(Debug, Clone, Default)]
pub struct Headers {
    inner: HashMap<String, String>,
}

impl Headers {
    pub fn new() -> Self {
        Self { inner: HashMap::new() }
    }

    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.inner.insert(key.into(), value.into());
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.inner.get(key)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.inner.contains_key(key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.inner.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn content_type(&self) -> Option<&str> {
        self.get("content-type").map(|s| s.as_str())
    }

    pub fn content_length(&self) -> Option<usize> {
        self.get("content-length").and_then(|s| s.parse().ok())
    }
}