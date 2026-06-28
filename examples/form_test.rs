use tezweb::{TezWeb, Response};

#[tokio::main]
async fn main() {
    TezWeb::new()
        .port(8080)
        .post("/login", |req, _| async move {
            let username = req.form("username").unwrap_or_default();
            let password = req.form("password").unwrap_or_default();
            Response::ok().json(&serde_json::json!({
                "username": username,
                "password": password,
                "status": "logged in!"
            }))
        })
        .run()
        .await
        .unwrap();
}