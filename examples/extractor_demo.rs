//! TezWeb Extractor Demo — without macro, direct usage
use tezweb::{TezWeb, Response};
use tezweb::extractors::{FromRequest, Path, Json, Query};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct User { id: String, name: String }

#[derive(Deserialize)]
struct NewUser { name: String, email: String }

#[tokio::main]
async fn main() {
    TezWeb::new()
        .port(8080)
        .get("/users/:id", |req, params| {
            async move {
                match Path::<String>::extract(&req, &params) {
                    Some(id) => {
                        let user = User { id: id.0, name: "Akram".to_string() };
                        Response::ok().json(&user)
                    }
                    None => Response::bad_request().text("Missing id")
                }
            }
        })
        .get("/search", |req, _params| {
            async move {
                match Query::extract(&req, &_params) {
                    Some(q) => {
                        let query = q.get("q").cloned().unwrap_or_default();
                        let page = q.get("page").cloned().unwrap_or_else(|| "1".to_string());
                        Response::ok().text(format!("search: {}, page: {}", query, page))
                    }
                    None => Response::bad_request().text("Missing query")
                }
            }
        })
        .post("/users", |req, _params| {
            async move {
                match Json::<NewUser>::extract(&req, &_params) {
                    Some(body) => Response::ok().json(&serde_json::json!({
                        "status": "created", "name": body.name, "email": body.email
                    })),
                    None => Response::bad_request().text("Invalid JSON")
                }
            }
        })
        .run().await.unwrap();
}
