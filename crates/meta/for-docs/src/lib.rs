#![no_std]

//! ### Application-facing crates
//!
//! - [`sel4`]: Straightforward, pure-Rust bindings to the seL4 API.
//! - [`sel4_config`]: Macros and constants corresponding to the seL4 kernel configuration. Can be used by all targets (i.e. in all of: application code, build scripts, and build-time tools).
#![cfg_attr(
    not(target_arch = "x86_64"),
    doc = "
- [`sel4_platform_info`]: Constants corresponding to the contents of `platform_info.h`. Can be used by all targets.
"
)]
//! - [`sel4_sync`]: Synchronization constructs using seL4 IPC. Currently only supports notification-based mutexes.
//! - [`sel4_logging`]: Log implementation for the [`log`] crate.
//!
//! ### Example root task runtime (if configured)
//!
#![cfg_attr(
    feature = "minimal-root-task-runtime",
    doc = "
- [`sel4_minimal_root_task_runtime`]: A minimal runtime which only supports a single thread without unwinding and without a global allocator.
"
)]
#![cfg_attr(
    feature = "full-root-task-runtime",
    doc = "
- [`sel4_full_root_task_runtime`]: A featureful runtime which supports thread-local storage and unwinding, and provides a global allocator.
"
)]
#![cfg_attr(
    not(any(
        feature = "minimal-root-task-runtime",
        feature = "full-root-task-runtime",
    )),
    doc = "
- (not configured)
"
)]
//!
//! ### Other crates of interest
//!
//! - [`sel4_sys`]: Raw bindings to the seL4 API, generated from the libsel4 headers and interface definition files. The `sel4` crate's implementation is based on this crate.

// pub use sel4;
// pub use sel4_sys;
// pub use sel4_config;
// pub use sel4_sync;
// pub use sel4_logging;

// #[cfg(not(target_arch = "x86_64"))]
// pub use sel4_platform_info;

// #[cfg(feature = "minimal-root-task-runtime")]
// pub use sel4_minimal_root_task_runtime;

// #[cfg(feature = "full-root-task-runtime")]
// pub use sel4_full_root_task_runtime;
