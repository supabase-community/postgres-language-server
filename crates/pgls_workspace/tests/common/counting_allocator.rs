#![allow(dead_code, reason = "each test binary uses a different part")]

//! A global allocator that counts the bytes allocated, for the memory tests. Each test binary
//! has its own allocator, so a binary only counts what its tests allocate: keep one test per
//! binary.

use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::atomic::{AtomicUsize, Ordering},
};

pub struct Counting;

/// The bytes allocated now.
pub static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
/// The most bytes allocated at once since it was last reset.
pub static PEAK: AtomicUsize = AtomicUsize::new(0);

fn add(size: usize) {
    let now = ALLOCATED.fetch_add(size, Ordering::Relaxed) + size;
    PEAK.fetch_max(now, Ordering::Relaxed);
}

// SAFETY: forwards to the system allocator, and only counts the sizes.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            add(layout.size());
        }
        ptr
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc_zeroed(layout) };
        if !ptr.is_null() {
            add(layout.size());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        ALLOCATED.fetch_sub(layout.size(), Ordering::Relaxed);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new = unsafe { System.realloc(ptr, layout, new_size) };
        if !new.is_null() {
            ALLOCATED.fetch_sub(layout.size(), Ordering::Relaxed);
            add(new_size);
        }
        new
    }
}

pub fn mib(bytes: usize) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}
