//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

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
//!   used by all targets, on configurations where this file exists.
//! - [`sel4_sync`]: Synchronization constructs using seL4 IPC. Currently only supports
//!   notification-based mutexes.
//! - [`sel4_logging`]: [`Log`](log::Log) implementation for the [`log`] crate.
//! - [`sel4_externally_shared`]: Abstractions for interacting with data in shared memory.
//! - [`sel4_shared_ring_buffer`]: Implementation of shared data structures used in the [seL4 Device
//!   Driver Framework](https://github.com/au-ts/sddf).
//! - `sel4_async_*`: Crates for leveraging async Rust in seL4 userspace.
//!
//! ### Runtime crates
//!
//! - **Root task**:
//!   - [`sel4_root_task`]: A runtime for root tasks that supports thread-local storage and
//!     unwinding, and provides a global allocator.
//! - **seL4 Microkit**:
//!   - [`sel4_microkit`]: A runtime for [seL4 Microkit](https://github.com/seL4/microkit)
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
    // sel4_abstract_rc
    sel4_async_block_io
    sel4_async_block_io_fat
    sel4_async_io
    sel4_async_network
    sel4_async_network_rustls
    // sel4_async_network_rustls_utils
    sel4_async_single_threaded_executor
    sel4_async_time
    sel4_async_unsync
    sel4_atomic_ptr
    sel4_bounce_buffer_allocator
    sel4_config
    sel4_dlmalloc
    sel4_driver_interfaces
    sel4_elf_header
    sel4_externally_shared
    sel4_immediate_sync_once_cell
    sel4_immutable_cell
    sel4_initialize_tls
    sel4_logging
    sel4_newlib
    sel4_one_ref_cell
    sel4_panicking
    sel4_panicking_env
    sel4_reset
    sel4_shared_ring_buffer
    sel4_shared_ring_buffer_block_io
    sel4_shared_ring_buffer_block_io_types
    sel4_shared_ring_buffer_bookkeeping
    sel4_shared_ring_buffer_smoltcp
    sel4_stack
    sel4_sync
    sel4_sync_trivial
    sel4_sys

    sel4_bcm2835_aux_uart_driver
    sel4_pl011_driver
    sel4_pl031_driver
    sel4_sp804_driver
    sel4_virtio_net
    sel4_virtio_blk
    sel4_virtio_hal_impl
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
