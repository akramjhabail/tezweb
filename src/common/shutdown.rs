//! Graceful Shutdown — Safe server stop
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use tokio::sync::broadcast;

static SHUTDOWN_FLAG: AtomicBool = AtomicBool::new(false);
static SHUTDOWN_TX: OnceLock<broadcast::Sender<()>> = OnceLock::new();

/// Shutdown system initialize karo
pub fn init_shutdown() -> broadcast::Receiver<()> {
    let tx = SHUTDOWN_TX.get_or_init(|| {
        let (tx, _) = broadcast::channel(1);
        tx
    });
    tx.subscribe()
}

/// Shutdown signal bhejo
pub fn trigger_shutdown() {
    SHUTDOWN_FLAG.store(true, Ordering::SeqCst);
    if let Some(tx) = SHUTDOWN_TX.get() {
        let _ = tx.send(());
    }
}

/// Server shutdown ho gaya?
pub fn is_shutdown() -> bool {
    SHUTDOWN_FLAG.load(Ordering::SeqCst)
}

/// Ctrl+C listener — background mein chalta hai
pub fn listen_for_ctrlc() {
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            println!("\n🛑 Shutting down TezWeb gracefully...");
            trigger_shutdown();
            // Thoda wait karo connections complete hon
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            println!("✅ TezWeb stopped. Bye! 👋");
            std::process::exit(0);
        }
    });
}