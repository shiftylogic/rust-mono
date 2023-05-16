/*++
 * Copyright (c) 2023-present Robert Anderson.
 * SPDX-License-Identifier: MIT
 *
 * Crate:   sl_core
 * Module:  allocators/tracing
 *
 * Purpose:
 *    Implements a wrapper allocator that logs all allocations with
 *    function information and allocation size.
 *
 *    Based on the following blog post:
 *     - https://shiver.github.io/post/tracking-heap-allocations-in-rust/
 *
 */

use std::{
    alloc::{
        GlobalAlloc,
        Layout,
    },
    cell::Cell,
};

thread_local! {
    static GUARD: Cell<bool> = Cell::new(false);
}

const UNKNOWN: &str = "<unknown>";
const IGNORED_SYMBOLS: [&str; 6] = [
    "_main",
    "__rg_alloc",
    "__rust_try",
    "backtrace::",
    "sl_core::allocators::tracing::trace_allocation",
    "<sl_core::allocators::tracing::Tracing",
];

pub struct Tracing<Allocator>(Allocator);

impl<Allocator> Tracing<Allocator> {
    pub const fn new(allocator: Allocator) -> Self { Self(allocator) }
}

unsafe impl<Allocator: GlobalAlloc> GlobalAlloc for Tracing<Allocator> {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.0.alloc(layout);
        trace_allocation(ptr, layout.size());
        ptr
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) { self.0.dealloc(ptr, layout); }
}

#[inline]
fn trace_allocation(ptr: *mut u8, size: usize) {
    GUARD.with(|guard| {
        if guard.get() {
            return;
        }

        guard.set(true);
        println!("------------------------------------------------------------");
        println!("Allocated {size} bytes @ Address {ptr:p}");
        backtrace::trace(|frame| {
            backtrace::resolve_frame(frame, |sym| {
                let fname = sym
                    .filename()
                    .map(|f| f.display().to_string())
                    .unwrap_or_default();
                if fname.starts_with("/rustc") {
                    return;
                }

                let sym_name = sym
                    .name()
                    .map(|n| n.to_string())
                    .unwrap_or(UNKNOWN.to_string());

                if IGNORED_SYMBOLS
                    .iter()
                    .any(|sym| sym_name.starts_with(sym) || sym_name.ends_with(sym))
                {
                    return;
                }

                let line_number = sym.lineno().unwrap_or(u32::MAX);

                println!("   > {sym_name} @ line {line_number}");
            });

            true
        });
        guard.set(false);
    });
}
