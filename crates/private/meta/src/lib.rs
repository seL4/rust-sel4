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
    {
        #[cfg($condition:meta)]
        $i:ident
    } => {
        #[cfg(not($condition))]
        use absent as $i;
        // #[doc(hidden)]
        #[cfg($condition)]
        pub use $i;
    };
}

macro_rules! definitely {
    ($($i:ident)*) => {
        $(
            // #[doc(hidden)]
            pub use $i;
        )*
    }
}

macro_rules! mutually_exclusive {
    ($tag:ident [$($feature:literal)*]) => {
        mod $tag {
            $(
                #[cfg(feature = $feature)]
                const $tag: () = ();
            )*
        }
    }
}

mutually_exclusive! {
    runtime_feature_check [
        "sel4-root-task"
        "sel4cp"
    ]
}

/// Placeholder for crates which are not part of this view.
pub mod absent {}

definitely! {
    sel4
    sel4_config
    sel4_sys
}

maybe! {
    #[cfg(all(
        feature = "sel4-platform-info",
        not(target_arch = "x86_64")
    ))]
    sel4_platform_info
}

maybe! {
    #[cfg(feature = "sel4-root-task")]
    sel4_root_task
}

maybe! {
    #[cfg(feature = "sel4cp")]
    sel4cp
}

definitely! {
    sel4_sync
    sel4_logging
    sel4_externally_shared
    sel4_shared_ring_buffer
    sel4_bounce_buffer_allocator
    sel4_async_network
    sel4_async_request_statuses
    sel4_async_single_threaded_executor
}
