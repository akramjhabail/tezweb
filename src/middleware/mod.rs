//! Middleware system for TezWeb

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use crate::http::Request;
use crate::http::Response;

pub mod rate_limit;
pub use rate_limit::{RateLimiter, rate_limit, rate_limit_middleware};

pub mod compression;
pub use compression::{compress, detect_encoding, Encoding};

pub mod timeout;
pub use timeout::{RequestTimeout, request_timeout};

pub mod body_limit;
pub use body_limit::{BodyLimit, body_limit};

// Middleware function types
pub type Next = Arc<dyn Fn(Request) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync>;

pub type MiddlewareFn = Arc<dyn Fn(Request, Next) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync>;

pub struct MiddlewareChain {
    pub middlewares: Vec<MiddlewareFn>,
}

impl MiddlewareChain {
    pub fn new() -> Self {
        Self { middlewares: Vec::new() }
    }

    pub fn add(&mut self, mw: MiddlewareFn) {
        self.middlewares.push(mw);
    }

    pub async fn run(&self, req: Request, handler: Next) -> Response {
        self.run_chain(req, handler, 0).await
    }

    fn run_chain(&self, req: Request, handler: Next, index: usize) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        if index >= self.middlewares.len() {
            return Box::pin(async move { handler(req).await });
        }

        let middleware = Arc::clone(&self.middlewares[index]);
        let chain = self.middlewares[index + 1..].to_vec();

        Box::pin(async move {
            let next: Next = Arc::new(move |req: Request| {
                let chain = chain.clone();
                let handler = Arc::clone(&handler);
                let remaining = MiddlewareChain { middlewares: chain };
                Box::pin(async move {
                    remaining.run(req, handler).await
                })
            });
            middleware(req, next).await
        })
    }
}

impl Default for MiddlewareChain {
    fn default() -> Self {
        Self::new()
    }
}

/// Logger middleware
pub fn logger() -> MiddlewareFn {
    Arc::new(|req, next| {
        Box::pin(async move {
            let method = format!("{:?}", req.method);
            let path = req.path.clone();
            let response = next(req).await;
            println!("[LOG] {} {} → {}", method, path, response.status);
            response
        })
    })
}

/// CORS middleware
/// Pass "*" to allow all, or a specific domain like "http://localhost:3000"
pub fn cors(allowed_origin: &str) -> MiddlewareFn {
    let origin = allowed_origin.to_string();
    Arc::new(move |req, next| {
        let origin = origin.clone();
        Box::pin(async move {
            let mut response = next(req).await;
            response.headers.push(("Access-Control-Allow-Origin".to_string(), origin));
            response.headers.push(("Access-Control-Allow-Methods".to_string(), "GET, POST, PUT, DELETE, OPTIONS".to_string()));
            response.headers.push(("Access-Control-Allow-Headers".to_string(), "Content-Type, Authorization".to_string()));
            response
        })
    })
}