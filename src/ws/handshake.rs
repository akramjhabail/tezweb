//! WebSocket HTTP → WS upgrade handshake

use sha1::{Sha1, Digest};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

const WS_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

pub struct WsHandshake {
    pub key: String,
}

impl WsHandshake {
    /// Request headers se WS key extract karo
    pub fn from_headers(headers: &[(String, String)]) -> Option<Self> {
        let key = headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("sec-websocket-key"))
            .map(|(_, v)| v.clone())?;

        // Upgrade: websocket check
        let is_upgrade = headers
            .iter()
            .any(|(k, v)| {
                k.eq_ignore_ascii_case("upgrade")
                    && v.eq_ignore_ascii_case("websocket")
            });

        if !is_upgrade { return None; }

        Some(Self { key })
    }

    /// Accept key generate karo
    pub fn accept_key(&self) -> String {
        let mut sha1 = Sha1::new();
        sha1.update(format!("{}{}", self.key, WS_GUID).as_bytes());
        BASE64.encode(sha1.finalize())
    }

    /// 101 Switching Protocols response banao
    pub fn response(&self) -> String {
        format!(
            "HTTP/1.1 101 Switching Protocols\r\n\
             Upgrade: websocket\r\n\
             Connection: Upgrade\r\n\
             Sec-WebSocket-Accept: {}\r\n\
             \r\n",
            self.accept_key()
        )
    }
}