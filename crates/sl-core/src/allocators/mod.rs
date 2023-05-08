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

mod counting;

pub use counting::Counting;
