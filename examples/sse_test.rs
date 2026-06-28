use tezweb::{TezWeb, Response};
use std::time::Duration;

#[tokio::main]
async fn main() {
    TezWeb::new()
        .port(8080)
        .get("/events", |_req, _params| async move {
            let (resp, stream) = Response::sse();

            // SSE stream background mein
            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                rt.block_on(async move {
                    for i in 1..=5 {
                        let msg = format!("Event number {}", i);
                        if !stream.send_data(&msg).await {
                            break;
                        }
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                });
            });

            resp
        })
        .run()
        .await
        .unwrap();
}