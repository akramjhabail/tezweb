//! SSE (Server-Sent Events) — live server → client updates

use tokio::sync::mpsc;

// ── Event ────────────────────────────────────────────────
pub struct SseEvent {
    pub event: Option<String>,
    pub data:  String,
    pub id:    Option<String>,
}

impl SseEvent {
    pub fn data(data: impl Into<String>) -> Self {
        Self { event: None, data: data.into(), id: None }
    }

    pub fn event(mut self, event: impl Into<String>) -> Self {
        self.event = Some(event.into());
        self
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn format(&self) -> String {
        let mut s = String::new();
        if let Some(id) = &self.id {
            s.push_str(&format!("id: {}\n", id));
        }
        if let Some(ev) = &self.event {
            s.push_str(&format!("event: {}\n", ev));
        }
        s.push_str(&format!("data: {}\n\n", self.data));
        s
    }
}

// ── Stream (sender side — handler ke paas) ──────────────
pub struct SseStream {
    pub(crate) sender: mpsc::Sender<SseEvent>,
}

impl SseStream {
    /// event naam ke saath data bhejo
    pub async fn send(&self, event: &str, data: &str) -> bool {
        self.sender
            .send(SseEvent::data(data).event(event))
            .await
            .is_ok()
    }

    /// sirf data bhejo (event naam nahi)
    pub async fn send_data(&self, data: &str) -> bool {
        self.sender.send(SseEvent::data(data)).await.is_ok()
    }

    /// heartbeat — connection zinda rakhne ke liye
    pub async fn ping(&self) -> bool {
        self.sender
            .send(SseEvent::data("ping").event("ping"))
            .await
            .is_ok()
    }
}

// ── Receiver (framework ke andar use hota hai) ───────────
pub struct SseReceiver {
    pub(crate) rx: mpsc::Receiver<SseEvent>,
}

/// ek channel banao — (Response ke liye receiver, handler ke liye stream)
pub fn sse_channel() -> (SseReceiver, SseStream) {
    let (tx, rx) = mpsc::channel(128);
    (SseReceiver { rx }, SseStream { sender: tx })
}