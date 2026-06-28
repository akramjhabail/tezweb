use tezweb::{TezWeb, Response, logger, cors};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[tokio::main]
async fn main() {
    TezWeb::new()
        .port(8080)
        .workers(4)
        .middleware(logger())
        .middleware(cors("*"))
        .get("/users/:id", |req, params| async move {
            let id: u32 = params["id"].parse().unwrap_or(0);

            // ✅ Query params
            let page  = req.query("page").unwrap_or("1");
            let limit = req.query("limit").unwrap_or("10");

            let user = User {
                id,
                name: format!("User {}", id),
                email: format!("user{}@example.com", id),
            };

            Response::ok().json(&serde_json::json!({
                "user": user,
                "page": page,
                "limit": limit,
            }))
        })
        .post("/users", |req, _params| async move {
            match req.json::<User>() {
                Ok(user) => Response::created().json(&user),
                Err(e) => Response::bad_request().json(&HashMap::from([
                    ("error", e.to_string())
                ]))
            }
        })
        .run()
        .await
        .unwrap();
}