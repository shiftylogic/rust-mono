/*++
 * Copyright (c) 2023-present Robert Anderson.
 * SPDX-License-Identifier: MIT
 *
 * Crate:   sl_core
 * Module:  allocators/tracing
 *
 * Purpose:
 *    Implements a wrapper allocator that tracks / logs all memory
 *    operations.
 *
 *    Idea started from the following blog post:
 *     - https://shiver.github.io/post/tracking-heap-allocations-in-rust/
 *
 */

use std::{
    alloc::{
        GlobalAlloc,
        Layout,
    },
    cell::RefCell,
    sync::Mutex,
};

pub use super::{
    DefaultTracker,
    Tracker,
};


thread_entry_guard!(TRACING_GUARD);


pub struct Tracing<A = std::alloc::System, T = DefaultTracker>
where
    A: GlobalAlloc,
    T: Tracker,
{
    inner:   A,
    tracker: Mutex<RefCell<T>>,
}

impl Tracing<std::alloc::System, DefaultTracker> {
    pub const fn default() -> Self {
        Self {
            inner:   std::alloc::System,
            tracker: Mutex::new(RefCell::new(DefaultTracker::new())),
        }
    }
}

impl<A, T> Tracing<A, T>
where
    A: GlobalAlloc,
    T: Tracker,
{
    pub const fn new(inner: A, tracker: T) -> Self {
        Self {
            inner,
            tracker: Mutex::new(RefCell::new(tracker)),
        }
    }

    pub fn dump_info<Writer: std::io::Write + ?Sized>(&self, out: &mut Writer, filter_std: bool) {
        no_reentry_per_thread!(TRACING_GUARD, {
            let tracker_guard = self.tracker.lock().expect("unable to unwrap tracker");
            (*tracker_guard)
                .borrow_mut()
                .dump_info(out, filter_std)
                .expect("failed to write tracker data");
        });
    }
}

unsafe impl<A, T> GlobalAlloc for Tracing<A, T>
where
    A: GlobalAlloc,
    T: Tracker,
{
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.inner.alloc(layout);

        no_reentry_per_thread!(TRACING_GUARD, {
            let tracker_guard = self.tracker.lock().expect("unable to unwrap tracker");
            (*tracker_guard).borrow_mut().track_alloc(ptr, layout);
        });

        ptr
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.inner.dealloc(ptr, layout);

        no_reentry_per_thread!(TRACING_GUARD, {
            let tracker_guard = self.tracker.lock().expect("unable to unwrap tracker");
            (*tracker_guard).borrow_mut().track_dealloc(ptr, layout);
        });
    }
}
