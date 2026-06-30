# Getting Started

TezWeb is a blazing-fast Rust HTTP framework built from scratch. No Hyper for HTTP/1.1, no Tower — just raw TCP and pure Rust.

## Installation

Add TezWeb to your `Cargo.toml`:

```toml
[dependencies]
tezweb = { git = "https://github.com/akramjhabail/tezweb" }
tokio = { version = "1", features = ["full"] }
```

## Hello World

```rust
use tezweb::{TezWeb, Response};

#[tokio::main]
async fn main() {
    TezWeb::new()
        .port(8080)
        .get("/hello", |_req, _params| async move {
            Response::ok().text("Hello from TezWeb! ⚡")
        })
        .run()
        .await
        .unwrap();
}
```

Run it:

```bash
cargo run
```

Visit `http://localhost:8080/hello` in your browser.

## JSON Response

```rust
use tezweb::{TezWeb, Response};
use serde_json::json;

#[tokio::main]
async fn main() {
    TezWeb::new()
        .port(8080)
        .get("/users", |_req, _params| async move {
            Response::ok().json(&json!([
                {"id": 1, "name": "Akram"},
                {"id": 2, "name": "Ali"}
            ]))
        })
        .run()
        .await
        .unwrap();
}
```

## Next Steps

- [Routing](./routing.md) — URL params, wildcards
- [Middleware](./middleware.md) — CORS, Logger, Rate Limiting
- [WebSocket](./websocket.md) — Real-time communication
- [Deployment](./deployment.md) — Deploy to production
