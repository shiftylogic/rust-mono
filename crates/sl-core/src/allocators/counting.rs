/*++
 *
 * Crate:   sl_core
 * Module:  allocators/counting
 *
 * Purpose:
 *   Set of custom allocators tailored to specific scenerios.
 *
 */

use std::{
    alloc::{GlobalAlloc, Layout},
    sync::atomic::{AtomicUsize, Ordering},
};

pub struct Counting<Allocator>(Allocator, AtomicUsize, AtomicUsize);

impl<Allocator> Counting<Allocator> {
    pub const fn new(allocator: Allocator) -> Self {
        Counting(allocator, AtomicUsize::new(0), AtomicUsize::new(0))
    }

    pub fn counts(&self) -> (usize, usize) {
        (
            self.1.load(Ordering::Relaxed),
            self.2.load(Ordering::Relaxed),
        )
    }
}

unsafe impl<Allocator: GlobalAlloc> GlobalAlloc for Counting<Allocator> {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.1.fetch_add(1, Ordering::Relaxed);
        self.2.fetch_add(1, Ordering::Relaxed);
        self.0.alloc(layout)
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        self.1.fetch_add(1, Ordering::Relaxed);
        self.2.fetch_add(1, Ordering::Relaxed);
        self.0.alloc_zeroed(layout)
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.2.fetch_sub(1, Ordering::Relaxed);
        self.0.dealloc(ptr, layout)
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        self.1.fetch_add(1, Ordering::Relaxed);
        self.0.realloc(ptr, layout, new_size)
    }
}
