mod ipc_buffer;
mod misc;
mod syscalls;

use ipc_buffer::{get_ipc_buffer, get_ipc_buffer_mut};

pub use ipc_buffer::{get_ipc_buffer_ptr, set_ipc_buffer_ptr};
pub use misc::*;
pub use syscalls::*;

include!(concat!(env!("OUT_DIR"), "/invocations.wrappers.rs"));
include!(concat!(env!("OUT_DIR"), "/shared_types.wrappers.rs"));
include!(concat!(env!("OUT_DIR"), "/types.wrappers.rs"));
