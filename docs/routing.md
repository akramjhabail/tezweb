# Routing

TezWeb uses a Trie-based router with O(log n) lookup — supports exact matches, URL parameters, and wildcards.

## Basic Routes

```rust
TezWeb::new()
    .get("/users", handler)
    .post("/users", handler)
    .put("/users/:id", handler)
    .delete("/users/:id", handler)
```

## URL Parameters

Use `:name` syntax to capture URL segments:

```rust
.get("/users/:id", |_req, params| async move {
    let id = params.get("id").cloned().unwrap_or_default();
    Response::ok().text(format!("User ID: {}", id))
})
```

Multiple params:

```rust
.get("/posts/:post_id/comments/:comment_id", |_req, params| async move {
    let post_id = &params["post_id"];
    let comment_id = &params["comment_id"];
    Response::ok().text(format!("Post {} Comment {}", post_id, comment_id))
})
```

## Wildcards

Use `*` to match any remaining path segments — useful for static files and proxying:

```rust
.get("/static/*", |_req, _params| async move {
    Response::ok().text("Matches /static/anything/here")
})
```

## Query Parameters

```rust
.get("/search", |req, _params| async move {
    let query = &req.query;
    Response::ok().text(format!("Query: {}", query))
})
```

## How It Works

TezWeb router is a custom Trie (prefix tree). Each segment of the URL path is a node. Exact matches are checked first, then param matches, then wildcard. This gives fast O(log n) lookup even with thousands of routes.
