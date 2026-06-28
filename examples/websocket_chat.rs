use tezweb::{TezWeb, Response, WsFrame, logger};

#[tokio::main]
async fn main() {
    TezWeb::new()
        .port(8080)
        .workers(4)
        .middleware(logger())
        // Normal HTTP route
        .get("/", |_req, _params| async {
            Response::ok().html(r#"
<!DOCTYPE html>
<html>
<head><title>TezWeb Chat</title></head>
<body>
    <h1>TezWeb WebSocket Chat</h1>
    <input id="msg" placeholder="Message..." />
    <button onclick="send()">Send</button>
    <div id="log"></div>
    <script>
        const ws = new WebSocket('ws://localhost:8080/ws');
        ws.onmessage = e => {
            document.getElementById('log').innerHTML += '<p>' + e.data + '</p>';
        };
        function send() {
            const msg = document.getElementById('msg').value;
            ws.send(msg);
        }
    </script>
</body>
</html>
            "#)
        })
        // WebSocket route
        .ws("/ws", |mut socket| async move {
            println!("Client connected!");
            let _ = socket.send_text("Welcome to TezWeb! 🚀").await;
    println!("🔥 WS HANDLER REACHED!");

            while let Some(frame) = socket.recv().await {
                match frame.opcode {
                    tezweb::ws::OpCode::Text => {
                        if let Some(text) = frame.text_str() {
                            println!("Received: {}", text);
                            let reply = format!("Echo: {}", text);
                            let _ = socket.send_text(reply).await;
                        }
                    }
                    tezweb::ws::OpCode::Close => {
                        println!("Client disconnected!");
                        socket.close().await;
                        break;
                    }
                    tezweb::ws::OpCode::Ping => {
                        let _ = socket.send(WsFrame::pong(frame.payload)).await;
                    }
                    _ => {}
                }
            }
        })
        .run()
        .await
        .unwrap();
}