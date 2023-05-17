/*++
 * Copyright (c) 2023-present Robert Anderson.
 * SPDX-License-Identifier: MIT
 *
 * Crate:   sl_core
 * Module:  allocators/tracker
 *
 * Purpose:
 *    Implements a "basic" tracker structure that keeps a log of all
 *    viewed allocations / de-allocations.
 *
 */

use std::{
    alloc::Layout,
    collections::{
        HashMap,
        HashSet,
    },
};

use backtrace::Backtrace;


//
// Tracker trait for overriding default memory tracking behaviors
//
pub trait Tracker {
    fn dump_info<Writer: std::io::Write + ?Sized>(
        &mut self,
        out: &mut Writer,
        filter_std: bool,
    ) -> std::io::Result<()>;

    fn track_alloc(&mut self, ptr: *mut u8, layout: Layout);
    fn track_dealloc(&mut self, ptr: *mut u8, layout: Layout);
}


//
// Default tracker tracked objects
//
enum Tracked {
    Allocation(usize, Layout, Backtrace),
    Deallocation(usize),
}


//
// "default" Tracker implementation
//
pub struct DefaultTracker {
    tracked: Vec<Tracked>,
}


impl DefaultTracker {
    pub const fn new() -> Self {
        Self {
            tracked: Vec::new(),
        }
    }
}


impl Tracker for DefaultTracker {
    fn dump_info<Writer: std::io::Write + ?Sized>(
        &mut self,
        out: &mut Writer,
        filter_std: bool,
    ) -> std::io::Result<()> {
        let mut leaked: HashMap<usize, (usize, usize)> = HashMap::new();
        let mut unknown_frees: HashSet<usize> = HashSet::new();

        for (idx, e) in self.tracked.iter_mut().enumerate() {
            match e {
                | Tracked::Allocation(ptr, layout, bt) => {
                    trace_allocation(out, idx, *ptr, layout, bt, filter_std)?;
                    leaked.insert(*ptr, (idx, layout.size()));
                },
                | Tracked::Deallocation(ptr) => {
                    if leaked.remove(ptr).is_none() {
                        unknown_frees.insert(*ptr);
                    }
                },
            }
        }

        if leaked.len() > 0 {
            writeln!(out, "\n\n=============== POSSIBLE LEAKS ===============")?;
            for (k, v) in leaked.iter() {
                writeln!(
                    out,
                    "[ID: {}] => {} bytes @ address {:p}",
                    v.0, v.1, *k as *const u8
                )?;
            }
        }

        if unknown_frees.len() > 0 {
            writeln!(out, "\n\n=============== UNKNOWN FREES ===============")?;
            for p in unknown_frees.iter() {
                writeln!(out, "  @ Address {:p}", *p as *const u8)?;
            }
        }


        Ok(())
    }

    fn track_alloc(&mut self, ptr: *mut u8, layout: Layout) {
        self.tracked.push(Tracked::Allocation(
            ptr as usize,
            layout,
            Backtrace::new_unresolved(),
        ));
    }

    fn track_dealloc(&mut self, ptr: *mut u8, _layout: Layout) {
        self.tracked.push(Tracked::Deallocation(ptr as usize));
    }
}


#[inline]
fn trace_allocation<Writer: std::io::Write + ?Sized>(
    out: &mut Writer,
    index: usize,
    ptr: usize,
    layout: &Layout,
    bt: &mut Backtrace,
    filter_std: bool,
) -> std::io::Result<()> {
    writeln!(
        out,
        "[ID: {}] ---------- Allocated {} bytes @ Address {:p} ----------",
        index,
        layout.size(),
        ptr as *const u8,
    )?;

    // These backtraces were not originally resolved.
    bt.resolve();

    for frame in bt.frames().iter() {
        for sym in frame.symbols() {
            if filter_std {
                let fname = sym
                    .filename()
                    .map(|f| f.display().to_string())
                    .unwrap_or_default();
                if fname.starts_with("/rustc") {
                    continue;
                }
            }

            let sym_name = sym
                .name()
                .map(|n| n.to_string())
                .unwrap_or(UNKNOWN.to_string());

            if IGNORED_SYMBOLS
                .iter()
                .any(|sym| sym_name.starts_with(sym) || sym_name.ends_with(sym))
            {
                continue;
            }

            let line_number = sym.lineno().unwrap_or(u32::MAX);

            writeln!(out, "   > {sym_name} @ line {line_number}")?;
        }
    }

    write!(out, "\n\n")?;
    Ok(())
}


const UNKNOWN: &str = "<unknown>";
const IGNORED_SYMBOLS: [&str; 4] = [
    "_main",
    "__rg_alloc",
    "backtrace::",
    "<sl_core::allocators::",
];
