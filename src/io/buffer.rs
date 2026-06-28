//! Zero-copy IO buffers using bytes crate

use bytes::{Bytes, BytesMut};
use std::ops::{Deref, DerefMut};

/// A zero-copy buffer for IO operations
pub struct IoBuffer {
    pub inner: BytesMut,
}

impl IoBuffer {
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            inner: BytesMut::with_capacity(cap),
        }
    }
    
    pub fn new() -> Self {
        Self::with_capacity(0)
    }
    
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
    
    pub fn as_ptr(&self) -> *const u8 {
        self.inner.as_ptr()
    }
    
    pub fn advance(&mut self, n: usize) {
        let _ = self.inner.split_to(n);
    }
    
    pub fn reserve(&mut self, n: usize) {
        self.inner.reserve(n);
    }
    
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }
    
    /// # Safety
    /// Pointer must be valid for `len` bytes.
    pub unsafe fn extend_from_ptr(&mut self, ptr: *const u8, len: usize) {
        self.inner.extend_from_slice(std::slice::from_raw_parts(ptr, len));
    }
    
    pub fn freeze(self) -> Bytes {
        self.inner.freeze()
    }
    
    pub fn clear(&mut self) {
        self.inner.clear();
    }
    
    pub fn split_to(&mut self, n: usize) -> Self {
        Self {
            inner: self.inner.split_to(n),
        }
    }
    
    pub fn split_off(&mut self, n: usize) -> Self {
        Self {
            inner: self.inner.split_off(n),
        }
    }
    
    pub fn to_vec(&self) -> Vec<u8> {
        self.inner.to_vec()
    }
}

impl Default for IoBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for IoBuffer {
    type Target = [u8];
    
    fn deref(&self) -> &[u8] {
        &self.inner
    }
}

impl DerefMut for IoBuffer {
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.inner
    }
}

impl From<Vec<u8>> for IoBuffer {
    fn from(vec: Vec<u8>) -> Self {
        Self {
            inner: BytesMut::from(&vec[..]),
        }
    }
}

impl From<&[u8]> for IoBuffer {
    fn from(slice: &[u8]) -> Self {
        Self {
            inner: BytesMut::from(slice),
        }
    }
}

impl AsRef<[u8]> for IoBuffer {
    fn as_ref(&self) -> &[u8] {
        &self.inner
    }
}

impl AsMut<[u8]> for IoBuffer {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.inner
    }
}