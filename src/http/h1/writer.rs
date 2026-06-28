//! HTTP/1.1 response writer with Keep-Alive support

use crate::io::IoBuffer;
use crate::sse::SseReceiver;

pub struct Response {
    pub status:     u16,
    pub headers:    Vec<(String, String)>,
    pub body:       Vec<u8>,
    pub keep_alive: bool,
    pub sse:        Option<SseReceiver>,
}

impl std::fmt::Debug for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Response")
            .field("status",     &self.status)
            .field("headers",    &self.headers)
            .field("body_len",   &self.body.len())
            .field("keep_alive", &self.keep_alive)
            .field("is_sse",     &self.sse.is_some())
            .finish()
    }
}

impl Response {
    pub fn new(status: u16) -> Self {
        Self {
            status,
            headers:    Vec::new(),
            body:       Vec::new(),
            keep_alive: true,
            sse:        None,
        }
    }

    pub fn ok()             -> Self { Self::new(200) }
    pub fn created()        -> Self { Self::new(201) }
    pub fn no_content()     -> Self { Self::new(204) }
    pub fn bad_request()    -> Self { Self::new(400) }
    pub fn not_found()      -> Self { Self::new(404) }
    pub fn internal_error() -> Self { Self::new(500) }

    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }

    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    pub fn keep_alive(mut self, enabled: bool) -> Self {
        self.keep_alive = enabled;
        self
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        let text = text.into();
        self.body = text.into_bytes();
        self.headers.push((
            "Content-Type".to_string(),
            "text/plain".to_string(),
        ));
        self
    }

    pub fn html(mut self, html: impl Into<String>) -> Self {
        let html = html.into();
        self.body = html.into_bytes();
        self.headers.push((
            "Content-Type".to_string(),
            "text/html".to_string(),
        ));
        self
    }

    pub fn json<T: serde::Serialize>(mut self, data: &T) -> Self {
        match serde_json::to_vec(data) {
            Ok(bytes) => {
                self.body = bytes;
                self.headers.push((
                    "Content-Type".to_string(),
                    "application/json".to_string(),
                ));
            }
            Err(_) => {
                self.status = 500;
                self.body = b"{\"error\":\"serialization failed\"}".to_vec();
                self.headers.push((
                    "Content-Type".to_string(),
                    "application/json".to_string(),
                ));
            }
        }
        self
    }

    pub fn sse() -> (Self, crate::sse::SseStream) {
        let (receiver, stream) = crate::sse::sse_channel();
        let mut resp = Self::new(200);
        resp.headers.push((
            "Content-Type".to_string(),
            "text/event-stream".to_string(),
        ));
        resp.headers.push((
            "Cache-Control".to_string(),
            "no-cache".to_string(),
        ));
        resp.headers.push((
            "X-Accel-Buffering".to_string(),
            "no".to_string(),
        ));
        resp.keep_alive = true;
        resp.sse = Some(receiver);
        (resp, stream)
    }

    pub fn write_sse_headers(&self, buf: &mut IoBuffer) {
        buf.inner.extend_from_slice(b"HTTP/1.1 200 OK\r\n");
        buf.inner.extend_from_slice(b"Connection: keep-alive\r\n");
        for (key, value) in &self.headers {
            if !key.eq_ignore_ascii_case("connection") {
                buf.inner.extend_from_slice(key.as_bytes());
                buf.inner.extend_from_slice(b": ");
                buf.inner.extend_from_slice(value.as_bytes());
                buf.inner.extend_from_slice(b"\r\n");
            }
        }
        buf.inner.extend_from_slice(b"\r\n");
    }

    pub fn status_text(status: u16) -> &'static str {
        match status {
            200 => "200 OK",
            201 => "201 Created",
            204 => "204 No Content",
            301 => "301 Moved Permanently",
            302 => "302 Found",
            400 => "400 Bad Request",
            401 => "401 Unauthorized",
            403 => "403 Forbidden",
            404 => "404 Not Found",
            405 => "405 Method Not Allowed",
            429 => "429 Too Many Requests",
            500 => "500 Internal Server Error",
            502 => "502 Bad Gateway",
            503 => "503 Service Unavailable",
            _   => "500 Internal Server Error",
        }
    }

    pub fn write_to(&self, buf: &mut IoBuffer) {
        buf.reserve(self.body.len() + 4096);

        buf.inner.extend_from_slice(b"HTTP/1.1 ");
        buf.inner.extend_from_slice(
            Self::status_text(self.status).as_bytes()
        );
        buf.inner.extend_from_slice(b"\r\n");

        if self.keep_alive {
            buf.inner.extend_from_slice(b"Connection: keep-alive\r\n");
            // ✅ timeout 60s, max 10000 (was 5s, 1000)
            buf.inner.extend_from_slice(
                b"Keep-Alive: timeout=60, max=10000\r\n"
            );
        } else {
            buf.inner.extend_from_slice(b"Connection: close\r\n");
        }

        let has_content_length = self.headers.iter()
            .any(|(k, _)| k.eq_ignore_ascii_case("content-length"));

        if !self.body.is_empty() && !has_content_length {
            buf.inner.extend_from_slice(b"Content-Length: ");
            buf.inner.extend_from_slice(
                self.body.len().to_string().as_bytes()
            );
            buf.inner.extend_from_slice(b"\r\n");
        }

        for (key, value) in &self.headers {
            if !key.eq_ignore_ascii_case("connection") {
                buf.inner.extend_from_slice(key.as_bytes());
                buf.inner.extend_from_slice(b": ");
                buf.inner.extend_from_slice(value.as_bytes());
                buf.inner.extend_from_slice(b"\r\n");
            }
        }

        buf.inner.extend_from_slice(b"\r\n");
        buf.inner.extend_from_slice(&self.body);
    }
}