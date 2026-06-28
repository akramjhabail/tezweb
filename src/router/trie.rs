//! Segment-based Trie router

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::http::h1::method::Method;
use crate::http::h1::parser::Request;
use crate::http::h1::writer::Response;
use crate::ws::WsHandler;

pub type Handler = Arc<dyn Fn(Request, HashMap<String, String>) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync>;

struct RouteNode {
    children: HashMap<String, RouteNode>,
    param_child: Option<(String, Box<RouteNode>)>,
    wildcard_handler: Option<Handler>,
    handlers: HashMap<u8, Handler>,
}

impl RouteNode {
    fn new() -> Self {
        Self {
            children: HashMap::new(),
            param_child: None,
            wildcard_handler: None,
            handlers: HashMap::new(),
        }
    }

    fn insert(&mut self, segments: &[&str], method: u8, handler: Handler) {
        if segments.is_empty() {
            self.handlers.insert(method, handler);
            return;
        }
        let seg  = segments[0];
        let rest = &segments[1..];
        if seg == "*" {
            self.wildcard_handler = Some(handler);
            return;
        }
        if let Some(name) = seg.strip_prefix(':') {
            if self.param_child.is_none() {
                self.param_child = Some((name.to_string(), Box::new(RouteNode::new())));
            }
            if let Some((_, node)) = &mut self.param_child {
                node.insert(rest, method, handler);
            }
        } else {
            self.children
                .entry(seg.to_string())
                .or_insert_with(RouteNode::new)
                .insert(rest, method, handler);
        }
    }

    fn find<'a>(
        &'a self,
        segments: &[&str],
        method: u8,
        params: &mut HashMap<String, String>,
    ) -> Option<&'a Handler> {
        // Pehle wildcard check — ye node kisi bhi remaining path ko match kar sakta hai
        if let Some(h) = &self.wildcard_handler {
            return Some(h);
        }
        if segments.is_empty() {
            return self.handlers.get(&method);
        }
        let seg  = segments[0];
        let rest = &segments[1..];
        // Exact match
        if let Some(child) = self.children.get(seg) {
            if let Some(h) = child.find(rest, method, params) {
                return Some(h);
            }
        }
        // Param match
        if let Some((param_name, child)) = &self.param_child {
            params.insert(param_name.clone(), seg.to_string());
            if let Some(h) = child.find(rest, method, params) {
                return Some(h);
            }
            params.remove(param_name);
        }
        None
    }
}

pub struct Router {
    root: RouteNode,
    ws_routes: HashMap<String, WsHandler>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            root: RouteNode::new(),
            ws_routes: HashMap::new(),
        }
    }

    pub fn add<F, Fut>(&mut self, method: Method, path: &str, handler: F)
    where
        F: Fn(Request, HashMap<String, String>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        let segments: Vec<&str> = path
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();
        let wrapped: Handler = Arc::new(move |req, params| {
            Box::pin(handler(req, params))
        });
        self.root.insert(&segments, method as u8, wrapped);
    }

    pub fn get<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(Request, HashMap<String, String>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.add(Method::GET, path, handler);
    }

    pub fn post<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(Request, HashMap<String, String>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.add(Method::POST, path, handler);
    }

    pub fn put<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(Request, HashMap<String, String>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.add(Method::PUT, path, handler);
    }

    pub fn delete<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(Request, HashMap<String, String>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.add(Method::DELETE, path, handler);
    }

    pub fn add_ws(&mut self, path: &str, handler: WsHandler) {
        self.ws_routes.insert(path.to_string(), handler);
    }

    pub fn find_ws(&self, path: &str) -> Option<WsHandler> {
        self.ws_routes.get(path).cloned()
    }

    pub fn find(
        &self,
        method: Method,
        path: &str,
    ) -> Option<(&Handler, HashMap<String, String>)> {
        let segments: Vec<&str> = path
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();
        let mut params = HashMap::new();
        self.root
            .find(&segments, method as u8, &mut params)
            .map(|h| (h, params))
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_handler() -> impl Fn(Request, HashMap<String, String>) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync + 'static {
        |_req, _params| Box::pin(async { Response::ok() })
    }

    #[test]
    fn test_exact_route() {
        let mut router = Router::new();
        router.get("/health", make_handler());
        assert!(router.find(Method::GET, "/health").is_some());
        assert!(router.find(Method::GET, "/other").is_none());
    }

    #[test]
    fn test_param_route() {
        let mut router = Router::new();
        router.get("/users/:id", make_handler());
        let (_, params) = router.find(Method::GET, "/users/42").unwrap();
        assert_eq!(params.get("id").unwrap(), "42");
    }

    #[test]
    fn test_wildcard_route() {
        let mut router = Router::new();
        router.get("/static/*", make_handler());
        assert!(router.find(Method::GET, "/static/index.html").is_some());
        assert!(router.find(Method::GET, "/static/css/style.css").is_some());
    }

    #[test]
    fn test_proxy_route() {
        let mut router = Router::new();
        router.get("/api/*", make_handler());
        assert!(router.find(Method::GET, "/api/health").is_some());
        assert!(router.find(Method::GET, "/api/users/123").is_some());
    }
}