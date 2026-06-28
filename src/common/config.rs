//! Configuration for TezWeb server

use std::time::Duration;

#[derive(Debug, Clone)]
pub struct TezConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub keep_alive_timeout: Duration,
    pub max_connections: usize,
    pub max_body_size: usize,
    pub tcp_nodelay: bool,
    pub http2_enabled: bool,
}

impl Default for TezConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            workers: num_cpus::get(),
            keep_alive_timeout: Duration::from_secs(5),
            max_connections: 10_000,
            max_body_size: 10 * 1024 * 1024,
            tcp_nodelay: true,
            http2_enabled: true,
        }
    }
}

impl TezConfig {
    pub fn new() -> Self { Self::default() }

    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into(); self
    }
    pub fn port(mut self, port: u16) -> Self {
        self.port = port; self
    }
    pub fn workers(mut self, workers: usize) -> Self {
        self.workers = workers; self
    }
    pub fn max_connections(mut self, max: usize) -> Self {
        self.max_connections = max; self
    }
    pub fn http2_enabled(mut self, enabled: bool) -> Self {
        self.http2_enabled = enabled; self
    }
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}