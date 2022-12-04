#![no_std]
#![feature(thread_local)]
#![feature(stmt_expr_attributes)]
#![feature(proc_macro_hygiene)]
#![feature(let_chains)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

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
