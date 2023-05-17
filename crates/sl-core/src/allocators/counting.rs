/*++
 * Copyright (c) 2022-present Robert Anderson.
 * SPDX-License-Identifier: MIT
 *
 * Crate:   sl_core
 * Module:  allocators/counting
 *
 * Purpose:
 *    Implements a simple allocator that counts the total number
 *    and active number of allocations.
 *
 */

use std::{
    alloc::{
        GlobalAlloc,
        Layout,
    },
    sync::atomic::{
        AtomicUsize,
        Ordering,
    },
};

pub struct Counting<A = std::alloc::System>
where
    A: GlobalAlloc,
{
    inner:  A,
    active: AtomicUsize,
    total:  AtomicUsize,
}

impl Counting<std::alloc::System> {
    pub const fn default() -> Self {
        Self {
            inner:  std::alloc::System,
            active: AtomicUsize::new(0),
            total:  AtomicUsize::new(0),
        }
    }
}

impl<A> Counting<A>
where
    A: GlobalAlloc,
{
    pub const fn new(inner: A) -> Self {
        Self {
            inner,
            active: AtomicUsize::new(0),
            total: AtomicUsize::new(0),
        }
    }

    pub fn counts(&self) -> (usize, usize) {
        (
            self.total.load(Ordering::Relaxed),
            self.active.load(Ordering::Relaxed),
        )
    }
}

unsafe impl<A> GlobalAlloc for Counting<A>
where
    A: GlobalAlloc,
{
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.total.fetch_add(1, Ordering::Relaxed);
        self.active.fetch_add(1, Ordering::Relaxed);
        self.inner.alloc(layout)
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.active.fetch_sub(1, Ordering::Relaxed);
        self.inner.dealloc(ptr, layout)
    }
}
