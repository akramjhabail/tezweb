use tezweb::{TezWeb, Response};

#[tokio::main]
async fn main() {
    println!("🔧 Backend server on port 9000");

    TezWeb::new()
        .port(9000)
        .get("/", |_req, _params| async move {
            Response::ok().text("Hello from TezWeb Backend!")
        })
        .get("/users", |_req, _params| async move {
            Response::ok().json(&serde_json::json!([
                {"id": 1, "name": "Akram"},
                {"id": 2, "name": "Ali"}
            ]))
        })
        .get("/users/:id", |_req, params| async move {
            let id = &params["id"];
            Response::ok().json(&serde_json::json!({"id": id, "name": "Akram"}))
        })
        .run()
        .await
        .unwrap();
}