use tezweb::{TezWeb, Response};

#[tokio::main]
async fn main() {
    TezWeb::new()
        .port(8080)
        .get("/", |_req, _params| async {
            Response::ok().text("File server coming soon!")
        })
        .run()
        .await
        .unwrap();
}