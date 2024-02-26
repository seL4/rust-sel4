use sel4::config::sel4_cfg_if;

sel4_cfg_if! {
    if #[sel4_cfg(PRINTING)] {
        pub use sel4_panicking_env::{debug_print, debug_println};
    } else {
        /// No-op for this configuration.
        #[macro_export]
        macro_rules! debug_print {
            ($($arg:tt)*) => {};
        }

        /// No-op for this configuration.
        #[macro_export]
        macro_rules! debug_println {
            ($($arg:tt)*) => {};
        }
    }
}

fn debug_put_char(c: u8) {
    sel4_cfg_if! {
        if #[sel4_cfg(PRINTING)] {
            sel4::debug_put_char(c)
        }
    }
}

sel4_panicking_env::register_debug_put_char!(debug_put_char);
