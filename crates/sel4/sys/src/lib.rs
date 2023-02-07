#![no_std]
#![feature(let_chains)]
#![feature(proc_macro_hygiene)]
#![feature(stmt_expr_attributes)]
#![feature(thread_local)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
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

#[cfg(feature = "wrappers")]
pub mod wrappers;

pub type ReplyAuthority = sel4_config::sel4_cfg_if! {
    if #[cfg(KERNEL_MCS)] {
        seL4_CPtr
    } else {
        ()
    }
};

pub type WaitMessageInfo = sel4_config::sel4_cfg_if! {
    if #[cfg(KERNEL_MCS)] {
        seL4_MessageInfo
    } else {
        ()
    }
};
