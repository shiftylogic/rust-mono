/*++
 * Copyright (c) 2023-present Robert Anderson.
 * SPDX-License-Identifier: MIT
 *
 * Crate:   sl_core
 * Module:  macros/thread
 *
 * Purpose:
 *    Macros specific to threading / guards / etc.
 *
 */


#[macro_export]
macro_rules! thread_entry_guard {
    ($name:ident) => {
        thread_local! { static $name: std::cell::Cell<bool> = std::cell::Cell::new(false); }
    };
}


#[macro_export]
macro_rules! no_reentry_per_thread {
    ($guard:ident, $body:block) => {
        $guard.with(|guard| {
            if guard.get() {
                return;
            }
            guard.set(true);
            $body
            guard.set(false);
        });
    };
}
