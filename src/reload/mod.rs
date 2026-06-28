//! Hot Reload — Built-in file watcher
//! Automatically restart server on file change

use notify::{Watcher, RecursiveMode, Event, EventKind};
use notify::event::{ModifyKind, CreateKind, RemoveKind};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Global reload flag
static RELOAD_NEEDED: AtomicBool = AtomicBool::new(false);

/// Reload zarurat hai?
pub fn is_reload_needed() -> bool {
    RELOAD_NEEDED.load(Ordering::Relaxed)
}

/// Reload flag reset karo
pub fn reset_reload() {
    RELOAD_NEEDED.store(false, Ordering::Relaxed);
}

/// File change event handle karo
fn should_reload(event: &Event) -> bool {
    matches!(&event.kind, EventKind::Modify(ModifyKind::Data(_)) | EventKind::Modify(ModifyKind::Name(_)) | EventKind::Create(CreateKind::File) | EventKind::Remove(RemoveKind::File))
}

/// Hot reload watcher start karo
pub fn start_watcher(watch_dirs: Vec<String>) {
    std::thread::spawn(move || {
        let last_reload = Arc::new(std::sync::Mutex::new(Instant::now()));
        let last_reload_c = Arc::clone(&last_reload);

        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                if should_reload(&event) {
                    // Debounce — 500ms mein ek hi reload
                    let mut last = last_reload_c.lock().unwrap();
                    if last.elapsed() > Duration::from_millis(500) {
                        *last = Instant::now();

                        let changed: Vec<String> = event.paths
                            .iter()
                            .filter_map(|p| p.to_str().map(|s| s.to_string()))
                            .collect();

                        for path in &changed {
                            println!("📝 File changed: {}", path);
                        }

                        RELOAD_NEEDED.store(true, Ordering::Relaxed);
                        println!("🔄 Hot reload triggered!");
                    }
                }
            }
        }).expect("Watcher banane mein error");

        for dir in &watch_dirs {
            let path = Path::new(dir);
            if path.exists() {
                match watcher.watch(path, RecursiveMode::Recursive) {
                    Ok(_)  => println!("👁️  Watching: {}", dir),
                    Err(e) => eprintln!("Watch error {}: {}", dir, e),
                }
            }
        }

        // Thread zinda rakho
        loop {
            std::thread::sleep(Duration::from_secs(1));
        }
    });
}

/// Default dirs watch karo
pub fn start_default_watcher() {
    let dirs = vec![
        "src".to_string(),
        "templates".to_string(),
        "public".to_string(),
        "static".to_string(),
    ];
    start_watcher(dirs);
    println!("🔥 Hot reload enabled!");
}