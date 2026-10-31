//! A global allocator that counts the bytes allocated by the test binary, for the memory tests.
//! Each test binary that uses it runs a single test, so that nothing else allocates at the same
//! time.
//!
//! Tree-sitter allocates its parsers and trees in C. [`count_native_allocations`] makes it
//! allocate through the counting allocator, so that the cached trees are counted too. Postgres'
//! parser also allocates in C, but only while it parses: the parse results that are cached are
//! Rust values.

// Each test binary uses a different part of it.
#![allow(dead_code)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::ffi::c_void;
use std::sync::Once;
use std::sync::atomic::{AtomicUsize, Ordering};

struct Counting;

pub static CURRENT: AtomicUsize = AtomicUsize::new(0);
pub static PEAK: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            let current = CURRENT.fetch_add(layout.size(), Ordering::Relaxed) + layout.size();
            PEAK.fetch_max(current, Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        CURRENT.fetch_sub(layout.size(), Ordering::Relaxed);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new = unsafe { System.realloc(ptr, layout, new_size) };
        if !new.is_null() {
            if new_size >= layout.size() {
                let grown = new_size - layout.size();
                let current = CURRENT.fetch_add(grown, Ordering::Relaxed) + grown;
                PEAK.fetch_max(current, Ordering::Relaxed);
            } else {
                CURRENT.fetch_sub(layout.size() - new_size, Ordering::Relaxed);
            }
        }
        new
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

pub const MIB: usize = 1024 * 1024;

pub fn mib(bytes: usize) -> f64 {
    bytes as f64 / MIB as f64
}

/// Resets the peak to the current usage and returns it.
pub fn reset_peak() -> usize {
    let current = CURRENT.load(Ordering::Relaxed);
    PEAK.store(current, Ordering::Relaxed);
    current
}

/// The bytes allocated right now.
pub fn current() -> usize {
    CURRENT.load(Ordering::Relaxed)
}

/// The most bytes allocated at once since the last [`reset_peak`].
pub fn peak() -> usize {
    PEAK.load(Ordering::Relaxed)
}

/// Makes tree-sitter allocate through the counting allocator. Call it before anything uses
/// tree-sitter: memory that it allocated before can't be freed with the new functions.
pub fn count_native_allocations() {
    static SET: Once = Once::new();
    // SAFETY: called before any tree-sitter allocation of the test, and the functions are a
    // complete malloc/calloc/realloc/free family.
    SET.call_once(|| unsafe {
        tree_sitter::set_allocator(
            Some(native_malloc),
            Some(native_calloc),
            Some(native_realloc),
            Some(native_free),
        );
    });
}

/// C's `free` gets no size, so each allocation starts with a header that holds its size, and
/// is aligned like `malloc`'s.
const HEADER: usize = 16;

fn native_layout(size: usize) -> Option<Layout> {
    Layout::from_size_align(size.checked_add(HEADER)?, HEADER).ok()
}

/// Writes the size into the header at `base`, and returns the memory after it.
unsafe fn with_header(base: *mut u8, size: usize) -> *mut c_void {
    if base.is_null() {
        return std::ptr::null_mut();
    }
    unsafe {
        base.cast::<usize>().write(size);
        base.add(HEADER).cast()
    }
}

/// The start of the allocation of `ptr`, and its size.
unsafe fn header(ptr: *mut c_void) -> (*mut u8, usize) {
    unsafe {
        let base = ptr.cast::<u8>().sub(HEADER);
        (base, base.cast::<usize>().read())
    }
}

unsafe extern "C" fn native_malloc(size: usize) -> *mut c_void {
    let Some(layout) = native_layout(size) else {
        return std::ptr::null_mut();
    };
    unsafe { with_header(std::alloc::alloc(layout), size) }
}

unsafe extern "C" fn native_calloc(count: usize, size: usize) -> *mut c_void {
    let Some(layout) = count.checked_mul(size).and_then(native_layout) else {
        return std::ptr::null_mut();
    };
    unsafe { with_header(std::alloc::alloc_zeroed(layout), layout.size() - HEADER) }
}

unsafe extern "C" fn native_realloc(ptr: *mut c_void, size: usize) -> *mut c_void {
    if ptr.is_null() {
        return unsafe { native_malloc(size) };
    }
    let (Some(new_layout), (base, old_size)) = (native_layout(size), unsafe { header(ptr) }) else {
        return std::ptr::null_mut();
    };
    let Some(old_layout) = native_layout(old_size) else {
        return std::ptr::null_mut();
    };
    unsafe {
        with_header(
            std::alloc::realloc(base, old_layout, new_layout.size()),
            size,
        )
    }
}

unsafe extern "C" fn native_free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let (base, size) = header(ptr);
        if let Some(layout) = native_layout(size) {
            std::alloc::dealloc(base, layout);
        }
    }
}
