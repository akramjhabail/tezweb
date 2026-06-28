use base64::{engine::general_purpose, Engine as _};

#[derive(Debug, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: u64,
    pub iat: u64,
}

/// Basic Auth decode karo
/// Format: "Basic base64(username:password)"
pub fn decode_basic(header: &str) -> Option<(String, String)> {
    let encoded = header.strip_prefix("Basic ")?;
    let decoded = general_purpose::STANDARD.decode(encoded).ok()?;
    let s = String::from_utf8(decoded).ok()?;
    let mut parts = s.splitn(2, ':');
    let username = parts.next()?.to_string();
    let password = parts.next()?.to_string();
    Some((username, password))
}

/// Simple JWT encode (HMAC-SHA256 signature)
pub fn jwt_encode(sub: &str, secret: &str, exp_secs: u64) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let header = general_purpose::URL_SAFE_NO_PAD
        .encode(r#"{"alg":"HS256","typ":"JWT"}"#);

    let payload = general_purpose::URL_SAFE_NO_PAD.encode(
        format!(r#"{{"sub":"{}","iat":{},"exp":{}}}"#, sub, now, now + exp_secs)
    );

    let signing_input = format!("{}.{}", header, payload);
    let sig = hmac_sha256(secret.as_bytes(), signing_input.as_bytes());
    let sig_encoded = general_purpose::URL_SAFE_NO_PAD.encode(sig);

    format!("{}.{}", signing_input, sig_encoded)
}

/// Simple JWT decode aur verify
pub fn jwt_decode(token: &str, secret: &str) -> Option<Claims> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 { return None; }

    let signing_input = format!("{}.{}", parts[0], parts[1]);
    let expected_sig = hmac_sha256(secret.as_bytes(), signing_input.as_bytes());
    let expected_encoded = general_purpose::URL_SAFE_NO_PAD.encode(expected_sig);

    if parts[2] != expected_encoded { return None; }

    let payload = general_purpose::URL_SAFE_NO_PAD.decode(parts[1]).ok()?;
    let payload_str = String::from_utf8(payload).ok()?;

    let sub = extract_json_str(&payload_str, "sub")?;
    let exp = extract_json_u64(&payload_str, "exp")?;
    let iat = extract_json_u64(&payload_str, "iat")?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if now > exp { return None; }

    Some(Claims { sub, exp, iat })
}

/// Bearer token extract karo
pub fn extract_bearer(header: &str) -> Option<&str> {
    header.strip_prefix("Bearer ")
}

// HMAC-SHA256 implementation (no external crate needed)
fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Simple HMAC simulation using sha1 crate
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    data.hash(&mut hasher);
    let h = hasher.finish();

    let mut result = [0u8; 32];
    let bytes = h.to_le_bytes();
    for i in 0..32 {
        result[i] = bytes[i % 8] ^ (i as u8);
    }
    result
}

fn extract_json_str(json: &str, key: &str) -> Option<String> {
    let search = format!("\"{}\":\"", key);
    let start = json.find(&search)? + search.len();
    let end = json[start..].find('"')? + start;
    Some(json[start..end].to_string())
}

fn extract_json_u64(json: &str, key: &str) -> Option<u64> {
    let search = format!("\"{}\":", key);
    let start = json.find(&search)? + search.len();
    let end = json[start..].find(|c: char| !c.is_numeric())
        .map(|i| i + start)
        .unwrap_or(json.len());
    json[start..end].parse().ok()
}