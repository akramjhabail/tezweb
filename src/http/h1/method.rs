//! HTTP methods

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum Method {
    #[default]
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
    CONNECT,
    TRACE,
}

impl Method {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GET => "GET",
            Self::POST => "POST",
            Self::PUT => "PUT",
            Self::DELETE => "DELETE",
            Self::PATCH => "PATCH",
            Self::HEAD => "HEAD",
            Self::OPTIONS => "OPTIONS",
            Self::CONNECT => "CONNECT",
            Self::TRACE => "TRACE",
        }
    }
    
    pub fn from_bytes(b: &[u8]) -> Option<Self> {
        match b {
            b"GET" => Some(Self::GET),
            b"POST" => Some(Self::POST),
            b"PUT" => Some(Self::PUT),
            b"DELETE" => Some(Self::DELETE),
            b"PATCH" => Some(Self::PATCH),
            b"HEAD" => Some(Self::HEAD),
            b"OPTIONS" => Some(Self::OPTIONS),
            b"CONNECT" => Some(Self::CONNECT),
            b"TRACE" => Some(Self::TRACE),
            _ => None,
        }
    }
    
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        Self::from_bytes(s.as_bytes())
    }
}

