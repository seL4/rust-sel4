cfg_if::cfg_if! {
    if #[cfg(feature = "alloc")] {
        mod with_alloc;
        use with_alloc as imp;
    } else {
        mod without_alloc;
        use without_alloc as imp;
    }
}

pub use imp::*;
