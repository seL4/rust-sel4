mod calls;
mod helpers;

pub use calls::*;

pub mod syscall_id {
    include!(concat!(env!("OUT_DIR"), "/syscall_ids.rs"));
}
