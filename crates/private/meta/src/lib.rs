#![no_std]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(unused_imports)]

//! ### General crates
//!
//! - [`sel4`]: Straightforward, pure-Rust bindings to the seL4 API.
//! - [`sel4_sys`]: Raw bindings to the seL4 API, generated from the libsel4 headers and interface
//!   definition files. This crate is not intended to be used directly by application code, but
//!   rather serves as a basis for the `sel4` crate's implementation.
//! - [`sel4_config`]: Macros and constants corresponding to the seL4 kernel configuration. Can be
//!   used by all targets (i.e. in all of: application code, build scripts, and build-time tools).
//! - [`sel4_platform_info`]: Constants corresponding to the contents of `platform_info.h`. Can be
//!   used by all targets.
//! - [`sel4_sync`]: Synchronization constructs using seL4 IPC. Currently only supports
//!   notification-based mutexes.
//! - [`sel4_logging`]: Log implementation for the [`log`] crate.
//! - [`sel4_externally_shared`]: Abstractions for interacting with data structures in shared
//!   memory.
//! - [`sel4_shared_ring_buffer`]: Implementation of shared data structures used in the [seL4 Device
//!   Driver Framework](https://github.com/lucypa/sDDF).
//! - `sel4_async_*`: Crates for leveraging async Rust in seL4 userspace.
//!
//! ### Runtime crates
//!
//! - **Root task**:
//!   - [`sel4_root_task`]: A runtime for root tasks that supports thread-local storage and
//!     unwinding, and provides a global allocator.
//! - **seL4 Microkit**:
//!   - [`sel4-microkit`]: A runtime for [seL4 Microkit](https://github.com/seL4/microkit)
//!     protection domains, including an implementation of libmicrokit and abstractions for IPC.

macro_rules! maybe {
    {
        #[cfg($condition:meta)]
        $i:ident
    } => {
        #[cfg(not($condition))]
        use absent as $i;
        #[cfg($condition)]
        pub use $i;
    };
}

macro_rules! definitely {
    ($($i:ident)*) => {
        $(
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
        "sel4-microkit"
    ]
}

/// Placeholder for crates which are not part of this view.
pub mod absent {}

definitely! {
    sel4
    sel4_async_block_io
    sel4_async_block_io_cpiofs
    sel4_async_network
    sel4_async_request_statuses
    sel4_async_single_threaded_executor
    sel4_async_timers
    sel4_bounce_buffer_allocator
    sel4_config
    sel4_externally_shared
    sel4_immediate_sync_once_cell
    sel4_immutable_cell
    sel4_logging
    sel4_shared_ring_buffer
    sel4_shared_ring_buffer_block_io
    sel4_shared_ring_buffer_block_io_types
    sel4_shared_ring_buffer_smoltcp
    sel4_sync
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
    #[cfg(feature = "sel4-microkit")]
    sel4_microkit
}

maybe! {
    #[cfg(feature = "sel4-microkit")]
    sel4_microkit_message
}

maybe! {
    #[cfg(feature = "sel4-microkit")]
    sel4_microkit_message_types
}
