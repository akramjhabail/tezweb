//! Memory management - Zero-copy, cache-aligned allocation
//!
//! ## Quick Start
//!
//! ```ignore
//! use crate::memory::*;
//!
//! let ptr = thread_bump_alloc!(128, 8);
//! let ptr = thread_bump_alloc_typed!(u64);
//! thread_bump_reset!();
//!
//! let counter = CacheAligned::new(0u64);
//! let padded  = CachePadded::new(0u64);
//!
//! let pool = ObjectPool::<String>::new();
//! pool.put(Box::new("hello".to_string()));
//! let obj = pool.get();
//! ```

pub mod bump;
pub mod pool;
pub mod cache_line;

pub use bump::BumpAllocator;
pub use pool::ObjectPool;
pub use cache_line::{CacheAligned, CachePadded};

/// Size of a single cache line (x86_64 / ARM64)
pub const CACHE_LINE_SIZE: usize = 64;

/// Target memory per request (bytes)
pub const REQUEST_MEMORY_TARGET: usize = 128;

/// Small allocation threshold (<1KB uses bump allocator)
pub const SMALL_ALLOC_THRESHOLD: usize = 1024;