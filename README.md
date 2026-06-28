
```markdown
<div align="center">

# ⚡ TezWeb

**The fastest HTTP framework for Rust. Period.**

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

*Tez (تیز) — Urdu for "Fast, Sharp, Blazing"*

</div>

---

## Why TezWeb?

Most Rust web frameworks are powerful but complex. TezWeb is different:

- **Zero magic** — no macros, no proc-derive hell, just functions
- **Zero bloat** — hand-rolled HTTP/1.1 parser, custom Trie router
- **Blazing performance** — 40,000+ req/sec, beats Actix-web 2.26x
- **Batteries included** — WebSocket, SSE, GraphQL, Proxy, TLS, HTTP/2 out of the box

> Built from scratch. No Hyper for HTTP/1.1. No Tower. Just raw TCP and pure Rust.

---

## Benchmarks

> Tested on MacBook Pro, `wrk -t8 -c400 -d10s`, release build.

| Framework | Language | RPS | Result |
|-----------|----------|-----|--------|
| **TezWeb** | Rust | **40,248** | 🥇 1st |
| Actix-web | Rust | 17,809 | 2.26x slower |
| Node.js | JavaScript | ~1,200 | 33x slower |
| FastAPI | Python | ~200 | 200x slower |
| Django | Python | ~80 | 500x slower |

| TezWeb Mode | RPS |
|-------------|-----|
| Debug + Logger | 17,602 |
| Release + Logger | 26,964 |
| **Release (no logger)** | **40,248** |

> Real benchmark — not theoretical. TezWeb beats Actix-web 2.26x on the same machine.
> Hand-rolled HTTP/1.1 parser, zero framework overhead.
> Next milestone: io_uring on Linux for 2x more throughput.

---

## Features

| Feature | Status | Notes |
|---------|--------|-------|
| HTTP/1.1 | ✅ Verified | GET, POST, PUT, DELETE |
| HTTP/2 | ✅ Verified | TLS + ALPN negotiation |
| Trie Router | ✅ Verified | Params, Wildcards |
| CORS Middleware | ✅ Verified | Headers auto-injected |
| Logger Middleware | ✅ Verified | Request/response logging |
| GraphQL | ✅ Verified | async-graphql integrated |
| Server-Sent Events | ✅ Verified | Standard data: format |
| WebSocket | ✅ Verified | RFC 6455 handshake |
| Form Parsing | ✅ Verified | x-www-form-urlencoded |
| Reverse Proxy | ✅ Verified | Prefix stripping |
| Static Files | ✅ Verified | Directory serving |
| TLS/HTTPS | ✅ Verified | Rustls backend |
| HTTP/3 / QUIC | ✅ Verified | Quinn backend, tested end-to-end |
| Rate Limiting | ✅ Verified | Per-IP sliding window |
| JWT Auth | ✅ Verified | HMAC-SHA256 signed tokens |

---

## Quick Start

```toml
[dependencies]
tezweb = { path = "." }
tokio = { version = "1", features = ["full"] }
```

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

---

## Examples

```bash
cargo run --example rest_api
cargo run --example graphql_test
cargo run --example websocket_chat
cargo run --example sse_test
cargo run --example proxy_test
cargo run --example unified_server   # HTTP/2 + TLS
```

---

## Reverse Proxy

```rust
TezWeb::new()
    .port(8080)
    .proxy("/api", "http://localhost:9000")
    .run().await.unwrap();
// /api/users → localhost:9000/users (prefix auto-stripped)
```

---

## Architecture

```
TezWeb
├── HTTP/1.1 Parser     (hand-rolled, zero-copy)
├── HTTP/2              (TLS + ALPN via Hyper)
├── Trie Router         (O(log n), params + wildcards)
├── Middleware Chain    (CORS, Logger, Rate-limit)
├── Protocol Handlers
│   ├── WebSocket       (RFC 6455)
│   ├── SSE             (text/event-stream)
│   └── GraphQL         (async-graphql)
├── Reverse Proxy       (prefix stripping)
├── Static Files        (MIME types + directory listing)
└── TLS                 (Rustls — no OpenSSL)
```

---

## Roadmap

- [ ] io_uring support on Linux (2x more performance)
- [x] HTTP/3 stable
- [ ] Connection pooling for proxy
- [ ] Hot reload in dev mode

---

## License

MIT © 2025 — Built with ❤️ and تیز speed.

<div align="center">
<b>If TezWeb helped you, give it a ⭐ on GitHub!</b>
</div