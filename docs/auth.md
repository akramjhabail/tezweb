# JWT Authentication

TezWeb provides built-in JWT (JSON Web Token) support using real HMAC-SHA256 cryptography.

## Generating a Token

```rust
use tezweb::auth::{generate_token, verify_token};

let token = generate_token("user123", "my-secret-key");
println!("Token: {}", token);
```

## Verifying a Token

```rust
match verify_token(&token, "my-secret-key") {
    Ok(claims) => println!("Valid! User: {}", claims),
    Err(e) => println!("Invalid token: {}", e),
}
```

## Using With Routes

```rust
TezWeb::new()
    .get("/login", |_req, _params| async move {
        let token = generate_token("user123", "secret-key");
        Response::ok().json(&serde_json::json!({ "token": token }))
    })
    .get("/protected", |req, _params| async move {
        let auth_header = req.headers.get("Authorization");
        match auth_header.and_then(|h| verify_token(h, "secret-key").ok()) {
            Some(_) => Response::ok().text("Access granted"),
            None => Response::new(401).text("Unauthorized"),
        }
    })
    .run()
    .await
    .unwrap();
```

## Security

TezWeb's JWT implementation uses the `hmac` and `sha2` crates for cryptographically secure HMAC-SHA256 signing — not a simple hash function. Tokens are tamper-proof: any modification to the payload invalidates the signature.

## Running the Example

```bash
cargo run --example jwt_test
```

## Verified

Tested end-to-end: token generation, valid-secret verification, wrong-secret rejection, and tampered-token rejection all pass correctly.