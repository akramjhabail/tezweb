## Rate Limiting

Limits requests per IP within a time window.

```rust
use tezweb::rate_limit_middleware;

TezWeb::new()
    .middleware(rate_limit_middleware(5, 10))
    .get("/api", handler)
```

Parameters: `rate_limit_middleware(max_requests, window_seconds)`.

Example above: max 5 requests per 10 seconds per IP. The 6th request within the window returns `429 Too Many Requests`.

The IP is read from the `X-Forwarded-For` header. If missing, a `"global"` key is used as fallback.

## Combining Middleware

Middleware runs in the order you add it:

```rust
TezWeb::new()
    .middleware(logger())
    .middleware(cors())
    .middleware(rate_limit_middleware(100, 60))
    .get("/api", handler)
    .run()
    .await
    .unwrap();
```

## Custom Middleware

You can write your own middleware function matching the `MiddlewareFn` signature to add custom behavior like authentication checks, request ID injection, or custom headers.