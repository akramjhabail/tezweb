//! HTTP/1.1 request parser with Keep-Alive & Chunked support

use httparse;
use crate::http::h1::method::Method;
use crate::io::IoBuffer;

#[derive(Debug)]
#[derive(Clone)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub query: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub keep_alive: bool,
    pub chunked: bool,
}

impl Request {
    pub fn new() -> Self {
        Self {
            method: Method::GET,
            path: String::new(),
            query: String::new(),
            headers: Vec::new(),
            body: Vec::new(),
            keep_alive: true,
            chunked: false,
        }
    }

    // ✅ Single query param
    pub fn query(&self, key: &str) -> Option<&str> {
        if self.query.is_empty() { return None; }
        for pair in self.query.split('&') {
            let mut parts = pair.splitn(2, '=');
            let k = parts.next().unwrap_or("");
            let v = parts.next().unwrap_or("");
            if k == key {
                return Some(v);
            }
        }
        None
    }

    // ✅ Sare query params as HashMap
    pub fn query_all(&self) -> std::collections::HashMap<String, String> {
        let mut map = std::collections::HashMap::new();
        if self.query.is_empty() { return map; }
        for pair in self.query.split('&') {
            let mut parts = pair.splitn(2, '=');
            let k = parts.next().unwrap_or("").to_string();
            let v = parts.next().unwrap_or("").to_string();
            if !k.is_empty() {
                map.insert(k, v);
            }
        }
        map
    }

    // ✅ JSON body parse
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(&self.body)
    }

    // ✅ Content-Type check
    pub fn is_json(&self) -> bool {
        self.headers
            .iter()
            .any(|(k, v)| {
                k.eq_ignore_ascii_case("content-type")
                    && v.to_lowercase().contains("application/json")
            })
    }

    // ✅ Form field — single value
    pub fn form(&self, key: &str) -> Option<String> {
        let body_str = std::str::from_utf8(&self.body).ok()?;
        for pair in body_str.split('&') {
            let mut parts = pair.splitn(2, '=');
            let k = parts.next().unwrap_or("");
            let v = parts.next().unwrap_or("");
            if k == key {
                return Some(url_decode(v));
            }
        }
        None
    }

    // ✅ Sare form fields as HashMap
    pub fn form_all(&self) -> std::collections::HashMap<String, String> {
        let mut map = std::collections::HashMap::new();
        let body_str = match std::str::from_utf8(&self.body) {
            Ok(s) => s,
            Err(_) => return map,
        };
        for pair in body_str.split('&') {
            let mut parts = pair.splitn(2, '=');
            let k = parts.next().unwrap_or("").to_string();
            let v = url_decode(parts.next().unwrap_or(""));
            if !k.is_empty() {
                map.insert(k, v);
            }
        }
        map
    }

    // ✅ Form content-type check
    pub fn is_form(&self) -> bool {
        self.headers
            .iter()
            .any(|(k, v)| {
                k.eq_ignore_ascii_case("content-type")
                    && v.to_lowercase().contains("application/x-www-form-urlencoded")
            })
    }

    pub fn parse(buf: &mut IoBuffer) -> Result<Option<Self>, u16> {
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut req = httparse::Request::new(&mut headers);

        match req.parse(buf) {
            Ok(httparse::Status::Complete(len)) => {
                let method = Method::from_bytes(req.method.unwrap().as_bytes())
                    .ok_or(400u16)?;

                let full_path = req.path.unwrap();
                let (path, query) = if let Some(pos) = full_path.find('?') {
                    (&full_path[..pos], &full_path[pos + 1..])
                } else {
                    (full_path, "")
                };

                let mut request = Request::new();
                request.method = method;
                request.path   = path.to_string();
                request.query  = query.to_string();

                for header in req.headers {
                    let key   = header.name.to_string();
                    let value = String::from_utf8_lossy(header.value).to_string();

                    if key.eq_ignore_ascii_case("connection") {
                        request.keep_alive = value.eq_ignore_ascii_case("keep-alive");
                    }
                    if key.eq_ignore_ascii_case("transfer-encoding")
                        && value.eq_ignore_ascii_case("chunked")
                    {
                        request.chunked = true;
                    }
                    request.headers.push((key, value));
                }

                buf.advance(len);

                if request.chunked {
                    request.parse_chunked_body(buf)?;
                } else {
                    request.parse_normal_body(buf)?;
                }

                Ok(Some(request))
            }
            Ok(httparse::Status::Partial) => Ok(None),
            Err(_) => Err(400u16),
        }
    }

    fn parse_normal_body(&mut self, buf: &mut IoBuffer) -> Result<(), u16> {
        let content_length = self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("content-length"))
            .and_then(|(_, v)| v.parse::<usize>().ok())
            .unwrap_or(0);

        if content_length > 0 && buf.len() >= content_length {
            self.body = buf[..content_length].to_vec();
            buf.advance(content_length);
        }
        Ok(())
    }

    fn parse_chunked_body(&mut self, buf: &mut IoBuffer) -> Result<(), u16> {
        let mut body = Vec::new();
        let data     = buf.to_vec();
        let mut pos  = 0;
        let data_len = data.len();

        loop {
            if pos >= data_len { return Ok(()); }

            let nl = data[pos..].iter().position(|&b| b == b'\n')
                .ok_or(400u16)?;

            let size_line = std::str::from_utf8(&data[pos..pos + nl])
                .map_err(|_| 400u16)?;

            let chunk_size = usize::from_str_radix(
                size_line.trim_end_matches('\r'), 16
            ).map_err(|_| 400u16)?;

            pos += nl + 1;

            if chunk_size == 0 {
                if pos < data_len && data[pos] == b'\r' { pos += 1; }
                if pos < data_len && data[pos] == b'\n' { pos += 1; }
                break;
            }

            if data_len < pos + chunk_size + 2 { return Ok(()); }

            body.extend_from_slice(&data[pos..pos + chunk_size]);
            pos += chunk_size;

            if pos < data_len && data[pos] == b'\r' { pos += 1; }
            if pos < data_len && data[pos] == b'\n' { pos += 1; }
        }

        buf.advance(pos);
        self.body = body;
        Ok(())
    }
}

impl Default for Request {
    fn default() -> Self {
        Self::new()
    }
}

// ✅ URL decode — %20 → space, + → space
fn url_decode(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'+' {
            result.push(' ');
            i += 1;
        } else if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(hex) = std::str::from_utf8(&bytes[i+1..i+3]) {
                if let Ok(val) = u8::from_str_radix(hex, 16) {
                    result.push(val as char);
                    i += 3;
                    continue;
                }
            }
            result.push(bytes[i] as char);
            i += 1;
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }
    result
}