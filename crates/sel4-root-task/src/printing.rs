//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4::config::sel4_cfg_if;

sel4_cfg_if! {
    if #[sel4_cfg(PRINTING)] {
        use sel4::debug_put_char;

        pub use sel4_panicking_env::{debug_print, debug_println};
    } else {
        fn debug_put_char(_: u8) {}

        /// No-op for this configuration
        #[macro_export]
        macro_rules! debug_print {
            ($($arg:tt)*) => {};
        }

        /// No-op for this configuration
        #[macro_export]
        macro_rules! debug_println {
            ($($arg:tt)*) => {};
        }
    }
}

sel4_panicking_env::register_debug_put_char!(debug_put_char);
