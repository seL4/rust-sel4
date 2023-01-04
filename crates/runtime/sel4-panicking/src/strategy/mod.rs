cfg_if::cfg_if! {
    if #[cfg(not(doc))] {
        #[cfg(all(feature = "force-abort", not(panic = "abort")))]
        compile_error!("feature = \"force-abort\" but not(panic = \"abort\"");

        #[cfg(all(feature = "force-unwind", not(panic = "unwind")))]
        compile_error!("feature = \"force-unwind\" but not(panic = \"unwind\"");
    }
}

cfg_if::cfg_if! {
    if #[cfg(all(panic = "unwind", feature = "allow-panic-unwind"))] {
        mod unwind;
        use unwind as imp;
    } else {
        mod abort;
        use abort as imp;
    }
}

pub(crate) use imp::*;
