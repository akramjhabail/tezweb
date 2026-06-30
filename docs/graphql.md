# GraphQL

TezWeb integrates with `async-graphql` for GraphQL support.

## Basic Setup

```rust
use tezweb::{TezWeb, graphql_handler};
use async_graphql::{Object, Schema, EmptyMutation, EmptySubscription};

struct Query;

#[Object]
impl Query {
    async fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }
}

#[tokio::main]
async fn main() {
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    TezWeb::new()
        .port(8080)
        .graphql("/graphql", schema)
        .run()
        .await
        .unwrap();
}
```

## Querying

Send a POST request to `/graphql`:

```bash
curl -X POST http://localhost:8080/graphql \
  -H "Content-Type: application/json" \
  -d '{"query":"{ add(a: 5, b: 7) }"}'
```

Response:

```json
{"data": {"add": 12}}
```

## Running the Example

```bash
cargo run --example graphql_test
```

## Verified

GraphQL query execution has been tested end-to-end. The example query `{ add(a: 5, b: 7) }` correctly returns `12`.