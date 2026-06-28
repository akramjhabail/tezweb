use std::collections::HashMap;
use std::pin::Pin;
use tezweb::{Router, Request, Response, Method};

// Helper function to create a mock handler for testing
fn mock_handler() -> impl Fn(Request, HashMap<String, String>) -> Pin<Box<dyn std::future::Future<Output = Response> + Send>> + Send + Sync + 'static {
    |_req, _params| Box::pin(async { Response::ok() })
}

#[test]
fn test_exact_match() {
    let mut router = Router::new();
    router.get("/health", mock_handler());
    
    assert!(router.find(Method::GET, "/health").is_some(), "Exact match '/health' failed");
    assert!(router.find(Method::GET, "/healthz").is_none(), "Should not match '/healthz'");
}

#[test]
fn test_param_route() {
    let mut router = Router::new();
    router.get("/users/:id", mock_handler());
    router.post("/users/:id/posts", mock_handler());

    // Test GET param
    let res = router.find(Method::GET, "/users/42");
    assert!(res.is_some(), "Param match '/users/42' failed");
    let (_, params) = res.unwrap();
    assert_eq!(params.get("id").unwrap(), "42", "Param 'id' should be 42");

    // Test POST param
    let res2 = router.find(Method::POST, "/users/1/posts");
    assert!(res2.is_some(), "Nested param match failed");
}

#[test]
fn test_404_not_found() {
    let mut router = Router::new();
    router.get("/api/status", mock_handler());
    
    assert!(router.find(Method::GET, "/api/wrong").is_none(), "Should return None for unknown routes");
    assert!(router.find(Method::POST, "/api/status").is_none(), "Method not allowed should return None");
}

#[test]
fn test_wildcard_route() {
    let mut router = Router::new();
    router.get("/static/*", mock_handler());
    
    assert!(router.find(Method::GET, "/static/index.html").is_some(), "Wildcard failed for index.html");
    assert!(router.find(Method::GET, "/static/css/style.css").is_some(), "Wildcard failed for nested path");
}
