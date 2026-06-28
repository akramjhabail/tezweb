use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub path: Option<String>,
    pub domain: Option<String>,
    pub max_age: Option<u64>,
    pub http_only: bool,
    pub secure: bool,
}

impl Cookie {
    pub fn new(name: &str, value: &str) -> Self {
        Self {
            name: name.to_string(),
            value: value.to_string(),
            path: Some("/".to_string()),
            domain: None,
            max_age: None,
            http_only: true,
            secure: false,
        }
    }

    pub fn http_only(mut self, v: bool) -> Self { self.http_only = v; self }
    pub fn secure(mut self, v: bool) -> Self { self.secure = v; self }
    pub fn max_age(mut self, secs: u64) -> Self { self.max_age = Some(secs); self }
    pub fn path(mut self, p: &str) -> Self { self.path = Some(p.to_string()); self }

    /// Cookie ko header string mein convert karo
    pub fn to_header(&self) -> String {
        let mut s = format!("{}={}", self.name, self.value);
        if let Some(ref p) = self.path    { s.push_str(&format!("; Path={}", p)); }
        if let Some(ref d) = self.domain  { s.push_str(&format!("; Domain={}", d)); }
        if let Some(age) = self.max_age   { s.push_str(&format!("; Max-Age={}", age)); }
        if self.http_only                 { s.push_str("; HttpOnly"); }
        if self.secure                    { s.push_str("; Secure"); }
        s
    }
}

/// Request se cookies parse karo
pub fn parse_cookies(header: &str) -> HashMap<String, String> {
    header.split(';')
        .filter_map(|pair| {
            let mut kv = pair.trim().splitn(2, '=');
            let k = kv.next()?.trim().to_string();
            let v = kv.next().unwrap_or("").trim().to_string();
            Some((k, v))
        })
        .collect()
}