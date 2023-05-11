sel4::config::sel4_cfg_if! {
    if #[cfg(PRINTING)] {
        pub use sel4_panicking_env::{debug_print, debug_println};
    } else {
        #[macro_export]
        macro_rules! debug_print {
            ($($arg:tt)*) => {};
        }

        #[macro_export]
        macro_rules! debug_println {
            ($($arg:tt)*) => {};
        }

        pub use debug_print;
        pub use debug_println;
    }
}
