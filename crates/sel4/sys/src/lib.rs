//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(thread_local)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(clippy::complexity)]
#![allow(clippy::new_without_default)]

mod bf;
mod c;
mod fault;
mod invocations;
mod ipc_buffer;
mod syscalls;

pub use bf::*;
pub use c::*;
pub use invocations::*;
pub use syscalls::*;

pub type ReplyAuthority = sel4_config::sel4_cfg_if! {
    if #[sel4_cfg(KERNEL_MCS)] {
        seL4_CPtr
    } else {
        ()
    }
};

pub type WaitMessageInfo = sel4_config::sel4_cfg_if! {
    if #[sel4_cfg(KERNEL_MCS)] {
        seL4_MessageInfo
    } else {
        ()
    }
};
