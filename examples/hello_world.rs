use tezweb::{TezWeb, Response};

#[tokio::main]
async fn main() {
    TezWeb::new()
        .port(std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(9000))
        .workers(4)
        .get("/", |_req, _params| async {
            Response::ok().text("Hello World!")
        })
        .get("/health", |_req, _params| async {
            Response::ok().text("OK")
        })
        .get("/users/:id", |_req, params| async move {
            let id = params.get("id").unwrap().clone();
            Response::ok().text(format!("User: {}", id))
        })
        .post("/users", |_req, _params| async {
            Response::ok().text("User created!")
        })
        .run()
        .await
        .unwrap();
}