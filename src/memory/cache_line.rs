//! Cache line aligned types for optimal CPU performance.
//!
//! FIXES:
//! - `CachePadded` rewritten: manual padding array removed.
//!   `repr(C, align(64))` already forces the compiler to pad the struct to a
//!   full cache line; an explicit `[u8; 64 - size % 64]` array caused:
//!   • a compile error when `size % 64 == 0` (array length = 64, not 0)
//!   • one extra cache line wasted for types whose size is already a
//!   multiple of 64
//!   The new definition is correct, minimal, and stable on Rust 1.56+.

use std::ops::{Deref, DerefMut};
use std::fmt;

// ── CacheAligned ─────────────────────────────────────────────────────────────

/// Forces `T` to start on a 64-byte cache-line boundary.
///
/// Useful to prevent **false sharing** when different threads access logically
/// independent data that happens to live on the same cache line.
///
/// Size: rounded up to the next multiple of 64 bytes by the compiler.
#[repr(C, align(64))]
pub struct CacheAligned<T> {
    value: T,
}

impl<T> CacheAligned<T> {
    #[inline]
    pub const fn new(value: T) -> Self {
        Self { value }
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.value
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T> Deref for CacheAligned<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T> DerefMut for CacheAligned<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T: Default> Default for CacheAligned<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Clone> Clone for CacheAligned<T> {
    fn clone(&self) -> Self {
        Self::new(self.value.clone())
    }
}

impl<T: fmt::Debug> fmt::Debug for CacheAligned<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<T: PartialEq> PartialEq for CacheAligned<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: Eq> Eq for CacheAligned<T> {}

// ── CachePadded ──────────────────────────────────────────────────────────────

/// Pads `T` so that the **whole struct** occupies at least one full cache line.
///
/// `repr(align(64))` tells the compiler to:
///   1. Align the struct to 64 bytes.
///   2. Round its size **up** to the next multiple of 64 bytes.
///
/// This means no two `CachePadded<T>` values (e.g. adjacent array elements)
/// share a cache line, eliminating false sharing without any manual padding.
///
/// ### Why not a manual `[u8; 64 - size_of::<T>() % 64]` array?
/// - When `size_of::<T>() % 64 == 0` the expression evaluates to `64`, adding
///   a whole extra cache line of wasted space.
/// - Stable Rust does not allow that expression in a const array-length
///   position for generic `T`, so it fails to compile.
/// - `repr(align(N))` achieves the same goal correctly and is stable since
///   Rust 1.0.
#[repr(C, align(64))]
pub struct CachePadded<T> {
    value: T,
    // No explicit padding field needed — the compiler inserts it automatically
    // to satisfy repr(align(64)).
}

impl<T> CachePadded<T> {
    #[inline]
    pub const fn new(value: T) -> Self {
        Self { value }
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.value
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T> Deref for CachePadded<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T> DerefMut for CachePadded<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T: Default> Default for CachePadded<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Clone> Clone for CachePadded<T> {
    fn clone(&self) -> Self {
        Self::new(self.value.clone())
    }
}

impl<T: fmt::Debug> fmt::Debug for CachePadded<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── CacheAligned ──────────────────────────────────────────────────────────

    #[test]
    fn cache_aligned_is_64_bytes_for_u32() {
        // u32 is 4 bytes; repr(align(64)) rounds the struct up to 64.
        assert_eq!(std::mem::size_of::<CacheAligned<u32>>(), 64);
    }

    #[test]
    fn cache_aligned_pointer_is_64_byte_aligned() {
        let a = CacheAligned::new(42u32);
        let ptr = &a as *const _ as usize;
        assert_eq!(ptr % 64, 0, "CacheAligned must be 64-byte aligned");
    }

    #[test]
    fn cache_aligned_deref_returns_inner() {
        let a = CacheAligned::new(99u64);
        assert_eq!(*a, 99u64);
    }

    #[test]
    fn cache_aligned_deref_mut_works() {
        let mut a = CacheAligned::new(1u32);
        *a.get_mut() = 42;
        assert_eq!(*a, 42);
    }

    // ── CachePadded ───────────────────────────────────────────────────────────

    #[test]
    fn cache_padded_size_is_multiple_of_64() {
        // Whatever T's size, CachePadded<T> must be a multiple of 64.
        assert_eq!(std::mem::size_of::<CachePadded<u8>>()   % 64, 0);
        assert_eq!(std::mem::size_of::<CachePadded<u64>>()  % 64, 0);
        assert_eq!(std::mem::size_of::<CachePadded<[u8;64]>>() % 64, 0);
        assert_eq!(std::mem::size_of::<CachePadded<[u8;65]>>() % 64, 0);
    }

    #[test]
    fn cache_padded_no_extra_waste_for_exact_multiples() {
        // A [u8; 64] inner value should NOT add another 64 bytes of padding.
        assert_eq!(std::mem::size_of::<CachePadded<[u8; 64]>>(), 64);
    }

    #[test]
    fn cache_padded_pointer_is_64_byte_aligned() {
        let p = CachePadded::new(7u8);
        let ptr = &p as *const _ as usize;
        assert_eq!(ptr % 64, 0, "CachePadded must be 64-byte aligned");
    }

    #[test]
    fn cache_padded_adjacent_elements_no_false_sharing() {
        // In an array, consecutive elements must be on separate cache lines.
        let arr = [CachePadded::new(0u64), CachePadded::new(1u64)];
        let p0 = &arr[0] as *const _ as usize;
        let p1 = &arr[1] as *const _ as usize;
        assert_eq!((p1 - p0) % 64, 0);
        assert!(p1 - p0 >= 64);
    }

    #[test]
    fn cache_padded_deref_returns_inner() {
        let p = CachePadded::new(123u32);
        assert_eq!(*p, 123u32);
    }
}