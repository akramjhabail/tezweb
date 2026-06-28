use tezweb::{TezWeb, Response};

#[tokio::main]
async fn main() {
    println!("🚀 Proxy test starting...");
    println!("📡 /api → http://localhost:9000");

    TezWeb::new()
        .port(8080)
        .proxy("/api", "http://localhost:9000")
        .get("/test", |_req, _params| async move {
            Response::ok().text("TezWeb working!")
        })
        .run()
        .await
        .unwrap();
}
