/*++
 * Copyright (c) 2022-present Robert Anderson.
 * SPDX-License-Identifier: MIT
 *
 * Crate:   sl_core
 * Module:  allocators
 *
 * Purpose:
 *   Set of custom allocators tailored to specific scenerios.
 *
 */

cfg_alloc_count! {
    mod counting;
    pub use counting::Counting;
}

cfg_alloc_trace! {
    mod tracing;
    pub use tracing::Tracing;
}
