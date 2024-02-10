//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

mod fault;
mod ipc_buffer;
mod misc;
mod syscalls;
mod tls;

use tls::{get_ipc_buffer, get_ipc_buffer_mut};

pub use fault::*;
pub use ipc_buffer::*;
pub use misc::*;
pub use syscalls::*;
pub use tls::{get_ipc_buffer_ptr, set_ipc_buffer_ptr};

include!(concat!(env!("OUT_DIR"), "/invocations.wrappers.rs"));
include!(concat!(env!("OUT_DIR"), "/shared_types.wrappers.rs"));
include!(concat!(env!("OUT_DIR"), "/types.wrappers.rs"));
