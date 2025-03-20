//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4::sel4_cfg_if;

sel4_cfg_if! {
    if #[sel4_cfg(PRINTING)] {
        use sel4::debug_put_char;

        pub use sel4_panicking_env::{debug_print, debug_println};
    } else {
        fn debug_put_char(_: u8) {}

        // Create new no-op macros instead of re-exporting from sel4_panicking_env for the sake of
        // performance.

        /// No-op for this configuration.
        #[macro_export]
        macro_rules! debug_print {
            ($($arg:tt)*) => {
                // Avoid unused argument warnings without runtime cost
                if false {
                    drop(format_args!($($arg)*))
                }
            };
        }

        /// No-op for this configuration.
        #[macro_export]
        macro_rules! debug_println {
            () => {
                ()
            };
            ($($arg:tt)*) => {
                // Avoid unused argument warnings without runtime cost
                if false {
                    drop(format_args!($($arg)*))
                }
            };
        }
    }
}

sel4_panicking_env::register_debug_put_char! {
    #[linkage = "weak"]
    debug_put_char
}
