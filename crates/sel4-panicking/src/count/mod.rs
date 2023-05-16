cfg_if::cfg_if! {
    if #[cfg(target_thread_local)] {
        mod with_tls;
        use with_tls as whether_tls;
    } else {
        mod without_tls;
        use without_tls as whether_tls;
    }
}

pub(crate) use whether_tls::*;
