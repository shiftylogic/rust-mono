/*++
 * Copyright (c) 2023-present Robert Anderson.
 * SPDX-License-Identifier: MIT
 *
 * Crate:   sl_core
 * Module:  macros/alloc
 *
 * Purpose:
 *    Macros specific to the memory allocators. These act as a convenience so
 *    we don't need a bunch of boilerplate for the general case.
 *
 */


cfg_alloc_count! {
    #[macro_export]
    macro_rules! enable_global_counting_alloc {
        () => {
            #[global_allocator]
            static GLOBAL: sl_core::allocators::Counting = sl_core::allocators::Counting::default();
        }
    }

    #[macro_export]
    macro_rules! trace_block {
        ( $tag:literal; $($t:tt)* ) => {
            let sl_sa = GLOBAL.counts();

            {
                $($t)*
            }

            let sl_ea = GLOBAL.counts();
            log::info!(
                "<{}> Allocations (Total {} => {} ({}), Active {} => {} ({}))",
                $tag,
                sl_sa.0,
                sl_ea.0,
                sl_ea.0 - sl_sa.0,
                sl_sa.1,
                sl_ea.1,
                sl_ea.1 - sl_sa.1,
            );
        }
    }

    #[macro_export]
    macro_rules! trace_fn {
        ( $f:expr ) => {
            {
                let sl_sa = GLOBAL.counts();
                let ret = $f();
                let sl_ea = GLOBAL.counts();
                log::info!(
                    "<{}> Allocations (Total {} => {} ({}), Active {} => {} ({}))",
                    stringify!($f),
                    sl_sa.0,
                    sl_ea.0,
                    sl_ea.0 - sl_sa.0,
                    sl_sa.1,
                    sl_ea.1,
                    sl_ea.1 - sl_sa.1,
                );
                ret
            }
        };

        ( $f:expr, $($params:tt)*? ) => {
            {
                let sl_sa = GLOBAL.counts();
                let ret = $f( $($params)* );
                let sl_ea = GLOBAL.counts();
                log::info!(
                    "<{}> Allocations (Total {} => {} ({}), Active {} => {} ({}))",
                    stringify!($f),
                    sl_sa.0,
                    sl_ea.0,
                    sl_ea.0 - sl_sa.0,
                    sl_sa.1,
                    sl_ea.1,
                    sl_ea.1 - sl_sa.1,
                );
                ret
            }
        }
    }
}


cfg_alloc_trace! {
    #[macro_export]
    macro_rules! enable_global_tracing_alloc {
        () => {
            #[global_allocator]
            static GLOBAL: sl_core::allocators::Tracing = sl_core::allocators::Tracing::default();
        }
    }

    #[macro_export]
    macro_rules! trace_block {
        ( $tag:literal; $($t:tt)* ) => {
            {
                $($t)*
            }

            let fname = std::fmt::format(format_args!(".{}_{}.log", $tag, std::process::id()));
            let mut mem_log = std::fs::File::create(fname).expect("failed to create mem log file");
            GLOBAL.dump_info(&mut mem_log);
        }
    }
}
