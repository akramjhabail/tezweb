# WebSocket

TezWeb implements RFC 6455 WebSocket with full handshake support.

## Basic WebSocket Server

```rust
use tezweb::{TezWeb, WsContext};

#[tokio::main]
async fn main() {
    TezWeb::new()
        .port(8080)
        .ws("/chat", |ctx: WsContext| async move {
            ctx.send("Welcome to TezWeb chat!").await;
        })
        .run()
        .await
        .unwrap();
}
```

## How It Works

1. Client sends an HTTP upgrade request to `/chat`
2. TezWeb performs the WebSocket handshake (SHA-1 key exchange per RFC 6455)
3. Connection upgrades from HTTP to WebSocket protocol
4. Your handler function runs with access to the `WsContext`

## Testing

Run the example:

```bash
cargo run --example websocket_chat
```

Connect using a WebSocket client (browser console):

```javascript
const ws = new WebSocket("ws://localhost:8080/chat");
ws.onmessage = (event) => console.log(event.data);
```

## Verified

WebSocket handshake and message handling have been tested end-to-end — handshake succeeds and the handler is reached correctly.