/*++
 * Copyright (c) 2023-present Robert Anderson.
 * SPDX-License-Identifier: MIT
 *
 * Crate:   sl_core
 * Module:  macros/cfg
 *
 * Purpose:
 *    Keep all configuration macros in a single place.
 *
 */

macro_rules! cfg_alloc_count {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "alloc-count")]
            $item
        )*
    }
}

macro_rules! cfg_alloc_trace {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "alloc-trace")]
            $item
        )*
    }
}
