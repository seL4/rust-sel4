use sel4_panicking_env::abort;

cfg_if::cfg_if! {
    if #[cfg(feature = "alloc")] {
        mod with_alloc;
        use with_alloc as whether_alloc;
    } else {
        mod without_alloc;
        use without_alloc as whether_alloc;
    }
}

pub(crate) use whether_alloc::*;

const RUST_EXCEPTION_CLASS: u64 = u64::from_be_bytes(*b"MOZ\0RUST");

pub(crate) fn drop_panic() -> ! {
    abort!("Rust panics must be rethrown")
}

pub(crate) fn foreign_exception() -> ! {
    abort!("Rust cannot catch foreign exceptions");
}
