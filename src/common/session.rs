use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

#[derive(Clone)]
struct SessionData {
    data: HashMap<String, String>,
    expires_at: Instant,
}

#[derive(Clone)]
pub struct SessionStore {
    store: Arc<RwLock<HashMap<String, SessionData>>>,
    ttl: Duration,
}

impl SessionStore {
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            ttl: Duration::from_secs(ttl_secs),
        }
    }

    /// Naya session banao — session ID return karta hai
    pub fn create(&self) -> String {
        let id = format!("sess_{}", rand_id());
        let mut store = self.store.write().unwrap();
        store.insert(id.clone(), SessionData {
            data: HashMap::new(),
            expires_at: Instant::now() + self.ttl,
        });
        id
    }

    /// Session se value lo
    pub fn get(&self, session_id: &str, key: &str) -> Option<String> {
        let store = self.store.read().unwrap();
        let session = store.get(session_id)?;
        if Instant::now() > session.expires_at { return None; }
        session.data.get(key).cloned()
    }

    /// Session mein value set karo
    pub fn set(&self, session_id: &str, key: &str, value: &str) -> bool {
        let mut store = self.store.write().unwrap();
        if let Some(session) = store.get_mut(session_id) {
            if Instant::now() <= session.expires_at {
                session.data.insert(key.to_string(), value.to_string());
                return true;
            }
        }
        false
    }

    /// Session delete karo
    pub fn destroy(&self, session_id: &str) {
        self.store.write().unwrap().remove(session_id);
    }

    /// Expired sessions clean karo
    pub fn cleanup(&self) {
        let now = Instant::now();
        self.store.write().unwrap().retain(|_, v| now <= v.expires_at);
    }
}

/// Simple random ID generator
fn rand_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    format!("{:x}", t)
}

/// Usage: SessionStore::new(3600) → 1 hour sessions
pub fn session_store(ttl_secs: u64) -> SessionStore {
    SessionStore::new(ttl_secs)
}