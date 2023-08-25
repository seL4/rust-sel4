#![no_std]
#![feature(cfg_target_thread_local)]
#![feature(exclusive_wrapper)]

#[cfg(feature = "start")]
mod start;

#[cfg(feature = "static-heap")]
mod static_heap;

#[cfg(any(feature = "tls", feature = "unwinding"))]
mod phdrs;

#[cfg(any(feature = "tls", feature = "unwinding"))]
pub use phdrs::*;

#[doc(hidden)]
pub mod _private {
    #[cfg(feature = "start")]
    pub use crate::start::_private as start;
    #[cfg(feature = "static-heap")]
    pub use crate::static_heap::_private as static_heap;
}
