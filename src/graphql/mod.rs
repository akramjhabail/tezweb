//! GraphQL — TezWeb
//! async-graphql integration with unique features

pub use async_graphql::{
    Schema, Object, SimpleObject, InputObject,
    Context, EmptyMutation, EmptySubscription,
    Result as GraphQLResult,
    http::GraphiQLSource,
};

use async_graphql::{Request as GqlRequest, Response as GqlResponse};
use crate::http::{Request, Response};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::Mutex;

// ── Query Analytics ──────────────────────────────────────
static TOTAL_QUERIES:    AtomicU64 = AtomicU64::new(0);
static FAILED_QUERIES:   AtomicU64 = AtomicU64::new(0);
static CACHE_HITS:       AtomicU64 = AtomicU64::new(0);
static CACHE_MISSES:     AtomicU64 = AtomicU64::new(0);
static PERSISTED_HITS:   AtomicU64 = AtomicU64::new(0);

pub fn get_query_stats() -> (u64, u64) {
    (
        TOTAL_QUERIES.load(Ordering::Relaxed),
        FAILED_QUERIES.load(Ordering::Relaxed),
    )
}

/// /graphql/stats endpoint
pub fn stats_response() -> Response {
    let total     = TOTAL_QUERIES.load(Ordering::Relaxed);
    let failed    = FAILED_QUERIES.load(Ordering::Relaxed);
    let cache_hit = CACHE_HITS.load(Ordering::Relaxed);
    let cache_miss= CACHE_MISSES.load(Ordering::Relaxed);
    let persisted = PERSISTED_HITS.load(Ordering::Relaxed);
    let success   = total.saturating_sub(failed);
    let hit_rate = (cache_hit * 100).checked_div(cache_hit + cache_miss).unwrap_or(0);

    let json = serde_json::json!({
        "tezweb_graphql_stats": {
            "total_queries":    total,
            "success_queries":  success,
            "failed_queries":   failed,
            "cache_hits":       cache_hit,
            "cache_misses":     cache_miss,
            "cache_hit_rate":   format!("{}%", hit_rate),
            "persisted_hits":   persisted,
        }
    });

    Response::ok()
        .header("Content-Type", "application/json")
        .body(serde_json::to_vec(&json).unwrap())
}

// ── Rate Limiter (per IP) ────────────────────────────────
pub struct GraphQLRateLimit {
    requests:       Mutex<HashMap<String, (u64, Instant)>>,
    max_per_minute: u64,
}

impl GraphQLRateLimit {
    pub fn new(max_per_minute: u64) -> Arc<Self> {
        Arc::new(Self {
            requests: Mutex::new(HashMap::new()),
            max_per_minute,
        })
    }

    pub fn check(&self, ip: &str) -> bool {
        let mut map = self.requests.lock().unwrap();
        let now   = Instant::now();
        let entry = map.entry(ip.to_string()).or_insert((0, now));
        if entry.1.elapsed() > Duration::from_secs(60) {
            *entry = (0, now);
        }
        entry.0 += 1;
        entry.0 <= self.max_per_minute
    }
}

// ── Query Cache ──────────────────────────────────────────
pub struct QueryCache {
    cache: Mutex<HashMap<String, (Vec<u8>, Instant)>>,
    ttl:   Duration,
}

impl QueryCache {
    pub fn new(ttl_secs: u64) -> Arc<Self> {
        Arc::new(Self {
            cache: Mutex::new(HashMap::new()),
            ttl:   Duration::from_secs(ttl_secs),
        })
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let cache = self.cache.lock().unwrap();
        if let Some((data, time)) = cache.get(key) {
            if time.elapsed() < self.ttl {
                return Some(data.clone());
            }
        }
        None
    }

    pub fn set(&self, key: String, value: Vec<u8>) {
        let mut cache = self.cache.lock().unwrap();
        cache.insert(key, (value, Instant::now()));
    }
}

// ── Persisted Queries ────────────────────────────────────
/// Query hash → actual query store karo
/// Client sirf hash bhejta hai — bandwidth bachao!
pub struct PersistedQueries {
    store: Mutex<HashMap<String, String>>,
}

impl PersistedQueries {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            store: Mutex::new(HashMap::new()),
        })
    }

    /// Hash se query nikalo
    pub fn get(&self, hash: &str) -> Option<String> {
        self.store.lock().unwrap().get(hash).cloned()
    }

    /// Query store karo — hash auto generate hoga
    pub fn register(&self, query: String) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        query.hash(&mut hasher);
        let hash = format!("{:x}", hasher.finish());
        self.store.lock().unwrap().insert(hash.clone(), query);
        hash
    }
}

// ── GraphQL Config ───────────────────────────────────────
pub struct GraphQLConfig {
    pub max_depth:       usize,
    pub max_complexity:  usize,
    pub rate_limit:      Option<Arc<GraphQLRateLimit>>,
    pub cache:           Option<Arc<QueryCache>>,
    pub persisted:       Option<Arc<PersistedQueries>>,
}

impl Default for GraphQLConfig {
    fn default() -> Self {
        Self {
            max_depth:      10,
            max_complexity: 100,
            rate_limit:     None,
            cache:          None,
            persisted:      None,
        }
    }
}

impl GraphQLConfig {
    pub fn new() -> Self { Self::default() }

    pub fn max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    pub fn max_complexity(mut self, complexity: usize) -> Self {
        self.max_complexity = complexity;
        self
    }

    pub fn rate_limit(mut self, max_per_minute: u64) -> Self {
        self.rate_limit = Some(GraphQLRateLimit::new(max_per_minute));
        self
    }

    pub fn cache(mut self, ttl_secs: u64) -> Self {
        self.cache = Some(QueryCache::new(ttl_secs));
        self
    }

    pub fn persisted_queries(mut self) -> Self {
        self.persisted = Some(PersistedQueries::new());
        self
    }
}

// ── Main Handler ─────────────────────────────────────────
pub async fn handle_graphql<Q, M, S>(
    req: Request,
    schema: &Schema<Q, M, S>,
) -> Response
where
    Q: async_graphql::ObjectType + 'static,
    M: async_graphql::ObjectType + 'static,
    S: async_graphql::SubscriptionType + 'static,
{
    handle_graphql_with_config(req, schema, &GraphQLConfig::new().cache(30)).await
}

/// Config ke saath GraphQL handle karo
pub async fn handle_graphql_with_config<Q, M, S>(
    req: Request,
    schema: &Schema<Q, M, S>,
    config: &GraphQLConfig,
) -> Response
where
    Q: async_graphql::ObjectType + 'static,
    M: async_graphql::ObjectType + 'static,
    S: async_graphql::SubscriptionType + 'static,
{
    TOTAL_QUERIES.fetch_add(1, Ordering::Relaxed);

    // ✅ Rate limit check
    if let Some(ref limiter) = config.rate_limit {
        let ip = req.headers.iter()
            .find(|(k, _)| k == "X-Forwarded-For")
            .map(|(_, v)| v.as_str())
            .unwrap_or("unknown");

        if !limiter.check(ip) {
            FAILED_QUERIES.fetch_add(1, Ordering::Relaxed);
            return Response::new(429).text("GraphQL rate limit exceeded");
        }
    }

    // Body parse karo
    let body = match std::str::from_utf8(&req.body) {
        Ok(s) => s.to_string(),
        Err(_) => {
            FAILED_QUERIES.fetch_add(1, Ordering::Relaxed);
            return Response::bad_request().text("Invalid UTF-8");
        }
    };

    // ✅ Persisted Query check
    // Client bhej sakta hai: {"extensions":{"persistedQuery":{"sha256Hash":"abc123"}}}
    let body = if let Ok(val) = serde_json::from_str::<serde_json::Value>(&body) {
        if let Some(hash) = val["extensions"]["persistedQuery"]["sha256Hash"].as_str() {
            if let Some(ref pq) = config.persisted {
                if let Some(query) = pq.get(hash) {
                    PERSISTED_HITS.fetch_add(1, Ordering::Relaxed);
                    // Hash se query restore karo
                    let mut new_val = val.clone();
                    new_val["query"] = serde_json::Value::String(query);
                    serde_json::to_string(&new_val).unwrap_or(body)
                } else {
                    return Response::bad_request()
                        .text(format!("Persisted query not found: {}", hash));
                }
            } else { body }
        } else { body }
    } else { body };

    let gql_req: GqlRequest = match serde_json::from_str(&body) {
        Ok(r) => r,
        Err(e) => {
            FAILED_QUERIES.fetch_add(1, Ordering::Relaxed);
            return Response::bad_request().text(format!("Invalid GraphQL: {}", e));
        }
    };

    // ✅ Cache check
    let cache_key = body.clone();
    if let Some(ref cache) = config.cache {
        if let Some(cached) = cache.get(&cache_key) {
            CACHE_HITS.fetch_add(1, Ordering::Relaxed);
            return Response::ok()
                .header("Content-Type", "application/json")
                .header("X-Cache", "HIT")
                .body(cached);
        }
    }

    // Execute karo
    let gql_resp: GqlResponse = schema.execute(gql_req).await;

    // Response serialize karo
    match serde_json::to_vec(&gql_resp) {
        Ok(bytes) => {
            CACHE_MISSES.fetch_add(1, Ordering::Relaxed);

            // Cache mein save karo
            if let Some(ref cache) = config.cache {
                cache.set(cache_key, bytes.clone());
            }

            Response::ok()
                .header("Content-Type", "application/json")
                .header("X-Cache", "MISS")
                .body(bytes)
        }
        Err(_) => {
            FAILED_QUERIES.fetch_add(1, Ordering::Relaxed);
            Response::internal_error().text("Response serialize failed")
        }
    }
}

/// GraphiQL UI
pub fn graphiql_response(endpoint: &str) -> Response {
    let html = GraphiQLSource::build()
        .endpoint(endpoint)
        .finish();
    Response::ok()
        .header("Content-Type", "text/html")
        .body(html.into_bytes())
}