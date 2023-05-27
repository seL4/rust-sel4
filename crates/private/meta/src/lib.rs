#![no_std]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(unused_imports)]

//! ### Application-facing crates
//!
//! - [`sel4`]: Straightforward, pure-Rust bindings to the seL4 API.
//! - [`sel4_config`]: Macros and constants corresponding to the seL4 kernel configuration. Can be used by all targets (i.e. in all of: application code, build scripts, and build-time tools).
//! - [`sel4_platform_info`]: Constants corresponding to the contents of `platform_info.h`. Can be used by all targets.
//! - [`sel4_sync`]: Synchronization constructs using seL4 IPC. Currently only supports notification-based mutexes.
//! - [`sel4_logging`]: Log implementation for the [`log`] crate.
//!
//! ### Runtimes
//!
//! - [`sel4_root_task`]: A runtime for root tasks which supports thread-local storage and unwinding, and provides a global allocator.
//! - [`sel4cp`]: A runtime for the seL4 Core Platform.
//!
//! ### Other crates of interest
//!
//! - [`sel4_sys`]: Raw bindings to the seL4 API, generated from the libsel4 headers and interface definition files. The [`sel4`] crate's implementation is based on this crate.

macro_rules! maybe {
    ($condition:meta, $i:ident) => {
        #[cfg(not($condition))]
        use absent as $i;
        #[doc(hidden)]
        #[cfg($condition)]
        pub use $i;
    };
}

macro_rules! mutually_exclusive {
    ($tag:ident, [$($feature:literal)*]) => {
        mod $tag {
            $(
                #[cfg(feature = $feature)]
                const $tag: () = ();
            )*
        }
    }
}

mutually_exclusive!(runtime_feature_check, [
    "sel4-root-task"
    "sel4cp"
]);

/// Placeholder for crates which are not part of this view.
pub mod absent {}

maybe!(target_env = "sel4", sel4);
maybe!(target_env = "sel4", sel4_config);
maybe!(target_env = "sel4", sel4_logging);
maybe!(target_env = "sel4", sel4_sync);
maybe!(target_env = "sel4", sel4_sys);
maybe!(
    all(
        target_env = "sel4",
        feature = "sel4-platform-info",
        not(target_arch = "x86_64")
    ),
    sel4_platform_info
);
maybe!(
    all(target_env = "sel4", feature = "sel4-root-task"),
    sel4_root_task
);
maybe!(all(target_env = "sel4", feature = "sel4cp"), sel4cp);
