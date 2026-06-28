//! Optimized event loop — Har OS pe fast!

use std::future::Future;

pub struct EventLoop {
    threads: usize,
}

impl EventLoop {
    pub fn new(threads: usize) -> Self {
        Self { threads }
    }

    pub fn run<F, Fut>(&self, f: F)
    where
        F: Fn() -> Fut + Send + Sync + Clone + 'static,
        Fut: Future<Output = ()>,
    {
        // ✅ Linux — io_uring (fastest)
        #[cfg(target_os = "linux")]
        {
            let mut handles = vec![];
            for _ in 0..self.threads {
                let f_clone = f.clone();
                handles.push(std::thread::spawn(move || {
                    monoio::RuntimeBuilder::<monoio::IoUringDriver>::new()
                        .with_entries(4096)
                        .enable_timer()
                        .build()
                        .unwrap()
                        .block_on(f_clone());
                }));
            }
            for h in handles {
                let _ = h.join();
            }
        }

        // ✅ Mac — kqueue optimized per core
        #[cfg(target_os = "macos")]
        {
            let mut handles = vec![];
            for _ in 0..self.threads {
                let f_clone = f.clone();
                handles.push(std::thread::spawn(move || {
                    tokio::runtime::Builder::new_current_thread()
                        .event_interval(31)
                        .global_queue_interval(31)
                        .max_blocking_threads(512)
                        .enable_all()
                        .build()
                        .unwrap()
                        .block_on(f_clone());
                }));
            }
            for h in handles {
                let _ = h.join();
            }
        }

        // ✅ Windows — IOCP thread per core
        #[cfg(target_os = "windows")]
        {
            let mut handles = vec![];
            for _ in 0..self.threads {
                let f_clone = f.clone();
                handles.push(std::thread::spawn(move || {
                    // ✅ Thread priority high karo
                    unsafe {
                        winapi::um::processthreadsapi::SetThreadPriority(
                            winapi::um::processthreadsapi::GetCurrentThread(),
                            2, // THREAD_PRIORITY_HIGHEST
                        );
                    }
                    tokio::runtime::Builder::new_current_thread()
                        .event_interval(31)
                        .global_queue_interval(31)
                        .max_blocking_threads(512)
                        .enable_all()
                        .build()
                        .unwrap()
                        .block_on(f_clone());
                }));
            }
            for h in handles {
                let _ = h.join();
            }
        }
    }

    pub fn run_single<F, Fut>(&self, f: F)
    where
        F: Fn() -> Fut + 'static,
        Fut: Future<Output = ()>,
    {
        #[cfg(target_os = "linux")]
        {
            monoio::RuntimeBuilder::<monoio::IoUringDriver>::new()
                .with_entries(4096)
                .enable_timer()
                .build()
                .unwrap()
                .block_on(f());
        }

        #[cfg(target_os = "macos")]
        {
            tokio::runtime::Builder::new_current_thread()
                .event_interval(31)
                .enable_all()
                .build()
                .unwrap()
                .block_on(f());
        }

        #[cfg(target_os = "windows")]
        {
            unsafe {
                winapi::um::processthreadsapi::SetThreadPriority(
                    winapi::um::processthreadsapi::GetCurrentThread(),
                    2, // THREAD_PRIORITY_HIGHEST
                );
            }
            tokio::runtime::Builder::new_current_thread()
                .event_interval(31)
                .enable_all()
                .build()
                .unwrap()
                .block_on(f());
        }
    }
}

impl Default for EventLoop {
    fn default() -> Self {
        Self::new(num_cpus::get())
    }
}