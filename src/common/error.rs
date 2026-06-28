//! TezWeb Error types

use std::fmt;

/// TezWeb ka unified error type — saare modules yahi use karte hain.
///
/// # Examples
/// ```
/// use tezweb::TezError;
/// let err = TezError::Internal("something broke".into());
/// assert_eq!(err.to_string(), "Internal error: something broke");
/// ```
#[derive(Debug)]
pub enum TezError {
    /// IO error (file, network, etc.)
    Io(std::io::Error),
    /// HTTP status code error
    Http(u16),
    /// HTTP/2 protocol error
    H2(h2::Error),
    /// Parsing error (WebSocket frame, HTTP header, etc.)
    Parse(String),
    /// JSON serialization/deserialization error
    Json(serde_json::Error),
    /// General timeout
    Timeout,
    /// Connection closed unexpectedly
    Closed,
    /// Internal framework error
    Internal(String),
    /// Rate limit exceeded
    RateLimitExceeded,
    /// Request-level timeout
    RequestTimeout,
    /// Request body exceeds configured limit
    BodyTooLarge { size: usize, limit: usize },
    /// WebSocket protocol error
    Ws(String),
}

/// Convenience alias for `Result<T, TezError>`
pub type TezResult<T> = Result<T, TezError>;

impl fmt::Display for TezError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e)       => write!(f, "IO error: {}", e),
            Self::Http(code)  => write!(f, "HTTP error: {}", code),
            Self::H2(e)       => write!(f, "H2 error: {}", e),
            Self::Parse(msg)  => write!(f, "Parse error: {}", msg),
            Self::Json(e)     => write!(f, "JSON error: {}", e),
            Self::Timeout     => write!(f, "Timeout"),
            Self::Closed      => write!(f, "Connection closed"),
            Self::Internal(m) => write!(f, "Internal error: {}", m),
            Self::RateLimitExceeded => write!(f, "Rate limit exceeded"),
            Self::RequestTimeout    => write!(f, "Request timeout"),
            Self::BodyTooLarge { size, limit } =>
                write!(f, "Body too large: {} bytes exceeds limit of {} bytes", size, limit),
            Self::Ws(msg)     => write!(f, "WebSocket error: {}", msg),
        }
    }
}

impl std::error::Error for TezError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e)   => Some(e),
            Self::H2(e)   => Some(e),
            Self::Json(e) => Some(e),
            _             => None,
        }
    }
}

impl From<std::io::Error> for TezError {
    fn from(e: std::io::Error) -> Self { Self::Io(e) }
}

impl From<h2::Error> for TezError {
    fn from(e: h2::Error) -> Self { Self::H2(e) }
}

impl From<serde_json::Error> for TezError {
    fn from(e: serde_json::Error) -> Self { Self::Json(e) }
}

impl From<String> for TezError {
    fn from(s: String) -> Self { Self::Internal(s) }
}
