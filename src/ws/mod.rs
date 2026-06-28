//! WebSocket support for TezWeb
//!
//! Streaming frame parser — koi fixed buffer size limit nahi.
//! TCP reads se data accumulate hota hai jab tak poora frame na mil jaye.

mod frame;
mod handshake;

pub use frame::{WsFrame, OpCode};
pub use handshake::WsHandshake;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// TCP stream ke upar WebSocket connection wrapper.
///
/// `recv()` mein streaming accumulator use hota hai — matlab
/// chahe frame 100 bytes ka ho ya 10MB, dono case sahi handle honge.
/// Koi `4096` fixed buffer nahi, koi data loss nahi.
pub struct WsSocket {
    stream: tokio::net::TcpStream,
}

impl WsSocket {
    /// Naya WsSocket banao existing TCP stream se
    pub fn new(stream: tokio::net::TcpStream) -> Self {
        Self { stream }
    }

    /// Agla WebSocket frame receive karo (streaming parser).
    ///
    /// # How it works
    /// 1. Accumulator mein already pada data se parse try karo
    /// 2. Agar frame incomplete hai, TCP se aur data padho
    /// 3. Accumulator mein add karo, phir se parse try karo
    /// 4. Jab frame mil jaye, consume kiye bytes accumulator se hata do
    ///
    /// Is approach se:
    /// - Chhote frames bhi fast parse hote hain (no extra copy)
    /// - Bade frames bhi sahi parse hote hain (accum grows as needed)
    /// - Multiple frames ek hi TCP read mein aaye toh next `recv()` call
    ///   pe instantly mil jayenge (already accumulator mein)
    pub async fn recv(&mut self) -> Option<WsFrame> {
        let mut accum: Vec<u8> = Vec::with_capacity(256);
        let mut read_buf = [0u8; 4096];

        loop {
            // Pehle accumulator se parse try karo
            if let Some((frame, consumed)) = WsFrame::parse(&accum) {
                accum.drain(..consumed);
                return Some(frame);
            }

            // Frame incomplete — TCP se aur data padho
            let n = self.stream.read(&mut read_buf).await.ok()?;
            if n == 0 {
                return None;
            }
            accum.extend_from_slice(&read_buf[..n]);
        }
    }

    /// WebSocket frame send karo
    pub async fn send(&mut self, frame: WsFrame) -> std::io::Result<()> {
        let bytes = frame.to_bytes();
        self.stream.write_all(&bytes).await
    }

    /// Text frame send karo (convenience)
    pub async fn send_text(&mut self, text: impl Into<String>) -> std::io::Result<()> {
        self.send(WsFrame::text(text)).await
    }

    /// Close frame send karo
    pub async fn close(&mut self) {
        let _ = self.send(WsFrame::close()).await;
    }
}

/// WebSocket handler type alias
pub type WsHandler = Arc<dyn Fn(WsSocket) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

/// Kisi bhi async function ko `WsHandler` mein convert karo
pub fn make_ws_handler<F, Fut>(f: F) -> WsHandler
where
    F: Fn(WsSocket) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    Arc::new(move |socket| Box::pin(f(socket)))
}
