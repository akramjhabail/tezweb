# Benchmarks

Real, measured performance — not theoretical.

## Test Setup

- Machine: MacBook Pro (Linux/macOS)
- Tool: `wrk -t8 -c400 -d10s`
- Build: `cargo build --release`
- Endpoint: `/health`

## Results

| Framework | RPS | Result |
|-----------|-----|--------|
| **TezWeb** | **40,248** | 1st place |
| Actix-web (8 workers) | 17,809 | 2.26x slower |

## TezWeb Across Modes

| Mode | RPS |
|------|-----|
| Debug build + Logger middleware | 17,602 |
| Release build + Logger middleware | 26,964 |
| Release build (no logger) | 40,248 |

Logger middleware adds noticeable overhead — for max throughput, disable it or use sampling in production.

## Reproducing the Benchmark

```bash
cd tezweb
cargo build --release --example rest_api
./target/release/examples/rest_api &
wrk -t8 -c400 -d10s http://localhost:8080/health
```

## Actix Comparison

```bash
cd actix-bench
cargo build --release
./target/release/actix-bench &
wrk -t8 -c400 -d10s http://localhost:8081/
```

## Why TezWeb Is Fast

1. Hand-rolled HTTP/1.1 parser — no Hyper overhead for HTTP/1.1
2. Custom Trie router — O(log n) lookup
3. Per-core TCP listeners with `SO_REUSEPORT`
4. Thread-local buffer pooling — reduces allocations
5. Zero unnecessary middleware by default

## Platform Notes

- **Linux**: Uses `monoio` + `io_uring` — expected to outperform macOS numbers shown here
- **macOS**: `kqueue` via tokio — current benchmark numbers (40,248 RPS)
- **Windows**: IOCP via tokio — untested