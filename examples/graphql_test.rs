use tezweb::TezWeb;
use tezweb::graphql::{Schema, Object, EmptyMutation, EmptySubscription};

struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn hello(&self) -> &str {
        "Hello from TezWeb GraphQL! 🚀"
    }

    async fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }

    async fn version(&self) -> &str {
        "TezWeb v0.1.0"
    }
}

#[tokio::main]
async fn main() {
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .finish();

    println!("📊 GraphiQL UI: http://localhost:8080/graphql");
    println!("✅ Cache: 30s TTL");
    println!("✅ Rate limit: 100 req/min per IP");
    println!("✅ Max depth: 5");

    TezWeb::new()
        .port(8080)
        .graphql("/graphql", schema)
        .run()
        .await
        .unwrap();
}