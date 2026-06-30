# Reverse Proxy

TezWeb supports reverse proxying with automatic prefix stripping.

## Basic Usage

```rust
use tezweb::TezWeb;

#[tokio::main]
async fn main() {
    TezWeb::new()
        .port(8080)
        .proxy("/api", "http://localhost:9000")
        .run()
        .await
        .unwrap();
}
```

## How Prefix Stripping Works

A request to `/api/users` is forwarded to the backend as `/users` — the `/api` prefix is stripped automatically.

| Incoming Request | Forwarded To |
|---|---|
| `/api/users` | `http://localhost:9000/users` |
| `/api/data` | `http://localhost:9000/data` |
| `/api/` | `http://localhost:9000/` |

## Multiple Proxy Rules

```rust
TezWeb::new()
    .proxy("/api", "http://localhost:9000")
    .proxy("/auth", "http://localhost:9001")
    .run()
    .await
    .unwrap();
```

## Testing

```bash
cargo run --example proxy_test
```

Then in another terminal:

```bash
python3 -m http.server 9000
curl http://localhost:8080/api/somefile
```

## Implementation Notes

The proxy opens a raw TCP connection to the target, forwards the HTTP request with the stripped path, and streams the response back to the client. Connection header is set to `close` for simplicity.

## Verified

Prefix stripping was tested end-to-end — `/api/somefile` correctly forwards to the backend as `/somefile`.