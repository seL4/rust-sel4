//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

//! This crate provides straightforward, pure-Rust bindings to the [seL4
//! API](https://sel4.systems/Info/Docs/seL4-manual-latest.pdf).
//!
//! Most items in this crate correspond to types, constants, and functions in
//! [libsel4](https://docs.sel4.systems/projects/sel4/api-doc.html). Notably, when applicable,
//! `seL4_CPtr` is agumented in [`LocalCPtr`] with a marker specifying the type of capability it
//! points to.
//!
//! This crate's implementation is based on the lower-level [`sel4-sys`](::sel4_sys) crate, which is
//! generated from the libsel4 headers and interface definition files.
//!
//! ### Features
//!
//! The `"state"` feature enables a thread-local `Option<RefCell<IPCBuffer>>` which, once set, in
//! turn enables threads to make seL4 API calls without having to explicitly specify an IPC buffer.
//! Specifically, it causes [`NoExplicitInvocationContext`] to be an alias for
//! [`ImplicitInvocationContext`], which implements [`InvocationContext`] by accessing the
//! thread-local pointer to an IPC buffer. When `"state"` is not set,
//! [`NoExplicitInvocationContext`] is an alias for [`NoInvocationContext`], which does not
//! implement [`InvocationContext`]. The thread-local IPC buffer pointer is modified and accessed by
//! the [`with_ipc_buffer`] family of functions.
//!
//! By default, `"state"` is implemented using `#[thread_local]`, and thus depends on ELF TLS. When
//! the feature `"single-threaded"` is enabled, this crate assumes that it will only be running in a
//! single thread, and instead implements `"state"` using a global `static`. This feature is useful
//! for runtimes where ELF TLS is not supported, but is only safe to use when this crate will only
//! be running in a single thread.
//!
//! ### Building
//!
//! This crate and its dependencies depend, at build time, on the libsel4 headers. The location of
//! these headers is provided to this crate at build time by environment variables. If
//! `SEL4_INCLUDE_DIRS` is set, then its value is interpreted as a colon-separated list of include
//! paths for the libsel4 headers. Otherwise, if `SEL4_PREFIX` is set, then
//! `$SEL4_PREFIX/libsel4/include` is used.

#![no_std]
#![feature(cfg_target_thread_local)]
#![feature(thread_local)]
#![allow(clippy::unit_arg)]

pub use sel4_config::{
    self as config, sel4_cfg, sel4_cfg_bool, sel4_cfg_enum, sel4_cfg_if, sel4_cfg_match,
    sel4_cfg_str, sel4_cfg_usize, sel4_cfg_wrap_match,
};

pub use sel4_sys as sys;

mod arch;
mod bootinfo;
mod cap_rights;
mod cnode_cap_data;
mod const_helpers;
mod cptr;
mod error;
mod helper_macros;
mod invocation_context;
mod invocations;
mod ipc_buffer;
mod message_info;
mod object;
mod reply_authority;
mod syscalls;
mod vspace;

pub mod fault;

pub use bootinfo::{BootInfo, BootInfoExtra, BootInfoExtraId, InitCSpaceSlot, UntypedDesc};
pub use cap_rights::{CapRights, CapRightsBuilder};
pub use cnode_cap_data::CNodeCapData;
pub use cptr::{
    cap_type, local_cptr, AbsoluteCPtr, CPtr, CPtrBits, CPtrWithDepth, CapType, HasCPtrWithDepth,
    LocalCPtr,
};
pub use error::{Error, Result};
pub use invocation_context::{
    ExplicitInvocationContext, InvocationContext, NoExplicitInvocationContext, NoInvocationContext,
};
pub use ipc_buffer::IPCBuffer;
pub use message_info::{MessageInfo, MessageInfoBuilder};
pub use object::{ObjectBlueprint, ObjectType};
pub use reply_authority::{ConveysReplyAuthority, ReplyAuthority};
pub use syscalls::{
    r#yield, Badge, CallWithMRs, FastMessages, IPCCapType, RecvWithMRs, NUM_MESSAGE_REGISTERS,
};
pub use vspace::{FrameType, SizedFrameType, GRANULE_SIZE};

sel4_cfg_if! {
    if #[cfg(KERNEL_MCS)] {
        pub use invocations::Time;
    } else {
        pub use syscalls::reply;
        pub use reply_authority::ImplicitReplyAuthority;
    }
}

pub use arch::top_level::*;

#[doc(no_inline)]
pub use local_cptr::*;

#[doc(no_inline)]
pub use fault::*;

pub(crate) use helper_macros::{
    declare_cap_type, declare_fault_newtype, declare_local_cptr_alias, newtype_methods,
};

sel4_cfg_if! {
    if #[cfg(DEBUG_BUILD)] {
        mod debug;
        mod fmt;

        pub use debug::{debug_put_char, debug_snapshot, get_clock};
        pub use fmt::DebugWrite;
    }
}

sel4_cfg_if! {
    if #[cfg(UINTR)] {
        pub use syscalls::wake_syscall_handler;
    }
}

sel4_cfg_if! {
    if #[cfg(ENABLE_BENCHMARKS)] {
        mod benchmark;

        pub use benchmark::{
            benchmark_reset_log,
            benchmark_finalize_log,
            benchmark_set_log_buffer,
        };

        sel4_cfg_if! {
            if #[cfg(BENCHMARK_TRACK_UTILISATION)] {
                pub use benchmark::{
                    benchmark_get_thread_utilisation,
                    benchmark_reset_thread_utilisation,
                };

                #[sel4_cfg(DEBUG_BUILD)]
                pub use benchmark::{
                    benchmark_dump_all_thread_utilisation,
                    benchmark_reset_all_thread_utilisation,
                };
            }
        }
    }
}

#[cfg(feature = "state")]
mod state;

#[cfg(feature = "state")]
pub use state::{set_ipc_buffer, with_ipc_buffer, with_ipc_buffer_mut, ImplicitInvocationContext};

/// Corresponds to `seL4_Word`.
pub type Word = sys::seL4_Word;

/// The size of [`Word`] in bits.
pub const WORD_SIZE: usize = sel4_cfg_usize!(WORD_SIZE);

#[doc(hidden)]
pub mod _private {
    #[sel4_config::sel4_cfg(DEBUG_BUILD)]
    pub use super::fmt::_private as fmt;
}
