use tezweb::{TezWeb, Response};

#[tokio::main]
async fn main() {
    TezWeb::new()
        .port(8080)
        .static_files("/static", "./public")
        .get("/", |_req, _params| async move {
            Response::ok().text("TezWeb is running!")
        })
        .run()
        .await
        .unwrap();
}