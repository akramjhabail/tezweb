//! Object pool for reusing expensive objects.

use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::ptr;

const POOL_SIZE: usize = 1024;

pub struct ObjectPool<T> {
    slots: [AtomicPtr<T>; POOL_SIZE],
    count: AtomicUsize,
}

impl<T> ObjectPool<T> {
    pub fn new() -> Self {
        Self {
            slots: [const { AtomicPtr::new(ptr::null_mut()) }; POOL_SIZE],
            count: AtomicUsize::new(0),
        }
    }

    pub fn get(&self) -> Option<Box<T>> {
        let mut idx = self.count.load(Ordering::Acquire);
        loop {
            if idx == 0 {
                return None;
            }
            match self.count.compare_exchange_weak(
                idx, idx - 1,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    let ptr = self.slots[idx - 1].swap(ptr::null_mut(), Ordering::AcqRel);
                    return if ptr.is_null() {
                        None
                    } else {
                        unsafe { Some(Box::from_raw(ptr)) }
                    };
                }
                Err(actual) => idx = actual,
            }
        }
    }

    pub fn put(&self, obj: Box<T>) {
        let mut idx = self.count.load(Ordering::Acquire);
        loop {
            if idx >= POOL_SIZE {
                return;
            }
            match self.count.compare_exchange_weak(
                idx, idx + 1,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    let ptr = Box::into_raw(obj);
                    let old = self.slots[idx].swap(ptr, Ordering::AcqRel);
                    // Agar slot mein kuch tha toh drop karo
                    if !old.is_null() {
                        unsafe { drop(Box::from_raw(old)) };
                    }
                    return;
                }
                Err(actual) => idx = actual,
            }
        }
    }

    pub fn clear(&self) {
        while self.get().is_some() {}
    }
}

impl<T> Default for ObjectPool<T> {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl<T: Send> Send for ObjectPool<T> {}
unsafe impl<T: Send> Sync for ObjectPool<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_reuses_objects() {
        let pool = ObjectPool::<String>::new();
        pool.put(Box::new("test".to_string()));
        let reused = pool.get();
        assert!(reused.is_some());
        assert_eq!(*reused.unwrap(), "test");
    }

    #[test]
    fn test_empty_pool_returns_none() {
        let pool = ObjectPool::<u64>::new();
        assert!(pool.get().is_none());
    }

    #[test]
    fn test_pool_capacity_limit() {
        let pool = ObjectPool::<u64>::new();
        for i in 0..POOL_SIZE {
            pool.put(Box::new(i as u64));
        }
        pool.put(Box::new(9999u64));
        let mut count = 0;
        while pool.get().is_some() {
            count += 1;
        }
        assert_eq!(count, POOL_SIZE);
    }

    #[test]
    fn test_clear_empties_pool() {
        let pool = ObjectPool::<String>::new();
        pool.put(Box::new("a".to_string()));
        pool.put(Box::new("b".to_string()));
        pool.clear();
        assert!(pool.get().is_none());
    }

    #[test]
    fn test_concurrent_put_get() {
        use std::sync::Arc;

        let pool = Arc::new(ObjectPool::<u64>::new());
        let mut handles = vec![];

        for t in 0..8u64 {
            let p = Arc::clone(&pool);
            handles.push(std::thread::spawn(move || {
                for i in 0..100u64 {
                    p.put(Box::new(t * 1000 + i));
                }
            }));
        }

        for _ in 0..8 {
            let p = Arc::clone(&pool);
            handles.push(std::thread::spawn(move || {
                for _ in 0..100 {
                    let _ = p.get();
                }
            }));
        }

        for h in handles {
            h.join().expect("thread panicked");
        }

        pool.clear();
    }
}