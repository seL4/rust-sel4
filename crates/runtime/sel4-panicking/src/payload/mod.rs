cfg_if::cfg_if! {
    if #[cfg(feature = "alloc")] {
        mod with_alloc;
        use with_alloc as whether_alloc;
    } else {
        mod without_alloc;
        use without_alloc as whether_alloc;
    }
}

pub use whether_alloc::*;
