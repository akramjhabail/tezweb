Yeh poora content paste karo `internals.md` mein:

```
# Internals

How TezWeb works under the hood.

## Architecture

```
TezWeb
├── HTTP/1.1 Parser     (hand-rolled, zero-copy)
├── HTTP/2              (TLS + ALPN via Hyper)
├── HTTP/3 / QUIC       (Quinn backend)
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

## HTTP/1.1 Parser

TezWeb does not use Hyper for HTTP/1.1. The parser is hand-rolled for zero-copy performance — it reads directly into a buffer pool and avoids unnecessary allocations.

## Trie Router

Routes are stored in a segment-based Trie (prefix tree). Each path segment (`/users`, `:id`, `*`) is a node. Lookup checks exact matches first, then parameter matches, then wildcards — giving O(log n) performance regardless of route count.

## Per-Core Architecture

TezWeb spawns one TCP listener per CPU core using `SO_REUSEPORT`, allowing the OS to load-balance incoming connections across cores without a single shared accept loop becoming a bottleneck.

## Platform-Specific I/O

- **Linux**: uses `monoio` with `io_uring` for kernel-level async I/O
- **macOS**: uses `tokio` with `kqueue`
- **Windows**: uses `tokio` with IOCP

## Buffer Pooling

A thread-local buffer pool reuses allocated buffers across requests instead of allocating fresh memory for every read/write, reducing GC-like pressure.

## TLS

TezWeb uses `rustls` exclusively — no OpenSSL dependency, which simplifies cross-platform builds and removes a common source of security vulnerabilities.
```

Pehle `Ctrl+A` (select all) → `Delete` → phir yeh paste karo → `Ctrl+S` save.