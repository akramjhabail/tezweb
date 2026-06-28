//! HTTP status codes

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusCode(pub u16);

impl StatusCode {
    pub const OK:                    Self = Self(200);
    pub const CREATED:               Self = Self(201);
    pub const NO_CONTENT:            Self = Self(204);
    pub const MOVED_PERMANENTLY:     Self = Self(301);
    pub const FOUND:                 Self = Self(302);
    pub const BAD_REQUEST:           Self = Self(400);
    pub const UNAUTHORIZED:          Self = Self(401);
    pub const FORBIDDEN:             Self = Self(403);
    pub const NOT_FOUND:             Self = Self(404);
    pub const METHOD_NOT_ALLOWED:    Self = Self(405);
    pub const TOO_MANY_REQUESTS:     Self = Self(429);
    pub const INTERNAL_SERVER_ERROR: Self = Self(500);
    pub const BAD_GATEWAY:           Self = Self(502);
    pub const SERVICE_UNAVAILABLE:   Self = Self(503);

    pub fn as_u16(&self) -> u16 { self.0 }

    pub fn as_str(&self) -> &'static str {
        match self.0 {
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

    pub fn is_success(&self)      -> bool { (200..300).contains(&self.0) }
    pub fn is_client_error(&self) -> bool { (400..500).contains(&self.0) }
    pub fn is_server_error(&self) -> bool { (500..600).contains(&self.0) }
}

impl From<u16> for StatusCode {
    fn from(code: u16) -> Self { Self(code) }
}

impl From<StatusCode> for u16 {
    fn from(code: StatusCode) -> Self { code.0 }
}