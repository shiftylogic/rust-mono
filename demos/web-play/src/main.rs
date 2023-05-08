/*++
 * Copyright (c) 2023-present Robert Anderson.
 * SPDX-License-Identifier: MIT
 *
 * Web playground / demo
 *
 */

use std::alloc;

use sl_core::allocators;

type Allocator = allocators::Counting<alloc::System>;

#[global_allocator]
static GLOBAL: Allocator = Allocator::new(alloc::System);

fn main() {
    env_logger::init();

    dump_allocations("entering <main> block");
    {
        match sl_web::run() {
            | Ok(_) => log::info!("Stopped."),
            | Err(e) => log::error!("[ERROR] {:?}", e),
        }
    }
    dump_allocations("exiting <main> block");
}

#[inline]
fn dump_allocations(tag: &str) {
    let x = GLOBAL.counts();
    log::info!("<{}> Allocations (Total: {}, Active: {})", tag, x.0, x.1);
}
