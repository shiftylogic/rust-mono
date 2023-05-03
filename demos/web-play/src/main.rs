/*++
 *
 * Web playground / demo
 *
 */

extern crate env_logger;
extern crate sl_core;
extern crate sl_web;

use std::alloc;

use sl_core::allocators;

type Allocator = allocators::Counting<alloc::System>;

#[global_allocator]
static GLOBAL: Allocator = Allocator::new(alloc::System);

fn main() {
    env_logger::init();

    dump_allocations("entering <main> block");
    {}
    dump_allocations("exiting <main> block");
}

#[inline]
fn dump_allocations(tag: &str) {
    let x = GLOBAL.counts();
    log::info!("<{}> Allocations (Total: {}, Active: {})", tag, x.0, x.1);
}
