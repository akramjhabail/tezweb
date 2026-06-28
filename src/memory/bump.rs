//! Bump allocator - O(1) allocations, no deallocation overhead

use std::cell::UnsafeCell;
use std::alloc::{Layout, alloc, dealloc};
use super::CACHE_LINE_SIZE;

const ARENA_SIZE: usize = 1024 * 1024;

thread_local! {
    pub static THREAD_BUMP: BumpAllocator = {
        let b = BumpAllocator::new();
        b.init();
        b
    };
}

#[macro_export]
macro_rules! thread_bump_alloc {
    ($size:expr, $align:expr) => {
        $crate::memory::bump::THREAD_BUMP.with(|b| b.alloc($size, $align))
    };
}

#[macro_export]
macro_rules! thread_bump_reset {
    () => {
        $crate::memory::bump::THREAD_BUMP.with(|b| b.reset())
    };
}

#[macro_export]
macro_rules! thread_bump_alloc_typed {
    ($T:ty) => {
        $crate::memory::bump::THREAD_BUMP.with(|b| b.alloc_typed::<$T>())
    };
}

struct FallbackAlloc {
    ptr: *mut u8,
    layout: Layout,
}

pub struct BumpAllocator {
    arena:     UnsafeCell<*mut u8>,
    offset:    UnsafeCell<usize>,
    fallbacks: UnsafeCell<Vec<FallbackAlloc>>,
}

impl Default for BumpAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl BumpAllocator {
    pub const fn new() -> Self {
        Self {
            arena:     UnsafeCell::new(std::ptr::null_mut()),
            offset:    UnsafeCell::new(0),
            fallbacks: UnsafeCell::new(Vec::new()),
        }
    }

    pub fn init(&self) {
        let layout = Layout::from_size_align(ARENA_SIZE, CACHE_LINE_SIZE)
            .expect("Invalid layout");
        let ptr = unsafe { alloc(layout) };
        assert!(!ptr.is_null(), "BumpAllocator: arena allocation failed");
        unsafe {
            *self.arena.get()  = ptr;
            *self.offset.get() = 0;
            (*self.fallbacks.get()).clear();
        }
    }

    #[inline(always)]
    pub fn alloc(&self, size: usize, align: usize) -> *mut u8 {
        debug_assert!(align.is_power_of_two());
        let offset = unsafe { &mut *self.offset.get() };
        let aligned_offset = (*offset + align - 1) & !(align - 1);
        let new_offset = aligned_offset + size;

        if new_offset <= ARENA_SIZE {
            *offset = new_offset;
            unsafe { (*self.arena.get()).add(aligned_offset) }
        } else {
            let layout = Layout::from_size_align(size, align)
                .expect("Invalid fallback layout");
            let ptr = unsafe { alloc(layout) };
            assert!(!ptr.is_null());
            unsafe {
                (*self.fallbacks.get()).push(FallbackAlloc { ptr, layout });
            }
            ptr
        }
    }

    #[inline(always)]
    pub fn alloc_typed<T>(&self) -> *mut T {
        self.alloc(std::mem::size_of::<T>(), std::mem::align_of::<T>()) as *mut T
    }

    #[inline(always)]
    pub fn reset(&self) {
        unsafe {
            let fallbacks = &mut *self.fallbacks.get();
            for fb in fallbacks.drain(..) {
                dealloc(fb.ptr, fb.layout);
            }
            *self.offset.get() = 0;
        }
    }
}

impl Drop for BumpAllocator {
    fn drop(&mut self) {
        let ptr = unsafe { *self.arena.get() };
        if !ptr.is_null() {
            let layout = Layout::from_size_align(ARENA_SIZE, CACHE_LINE_SIZE)
                .expect("Invalid layout");
            unsafe { dealloc(ptr, layout) };
        }
        self.reset();
    }
}

unsafe impl Send for BumpAllocator {}
unsafe impl Sync for BumpAllocator {}