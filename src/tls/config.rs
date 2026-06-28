// src/tls/config.rs

use std::path::PathBuf;

/// TLS Configuration
#[derive(Clone, Debug)]
pub struct TlsConfig {
    /// Certificate file path (.pem or .crt)
    pub cert_path: PathBuf,
    /// Private key file path (.pem or .key)
    pub key_path: PathBuf,
    /// HTTPS port (default: 443)
    pub port: u16,
    /// Auto redirect HTTP → HTTPS
    pub redirect_http: bool,
}

impl TlsConfig {
    /// Naya TLS config banao
    pub fn new(cert_path: impl Into<PathBuf>, key_path: impl Into<PathBuf>) -> Self {
        Self {
            cert_path: cert_path.into(),
            key_path: key_path.into(),
            port: 443,
            redirect_http: true,
        }
    }

    /// Custom port set karo
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// HTTP → HTTPS redirect on/off
    pub fn redirect_http(mut self, redirect: bool) -> Self {
        self.redirect_http = redirect;
        self
    }
}