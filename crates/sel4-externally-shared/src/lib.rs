//! Provides wrapper types for raw pointers to externally shared data.
//!
//! This crate provides two different wrapper types: [`ExternallySharedPtr`] and [`ExternallySharedRef`]. The
//! difference between the two types is that the former behaves like a raw pointer, while the
//! latter behaves like a Rust reference type.

#![no_std]
#![cfg_attr(feature = "unstable", feature(atomic_from_ptr))]
#![cfg_attr(feature = "unstable", feature(core_intrinsics))]
#![cfg_attr(feature = "unstable", feature(slice_range))]
#![cfg_attr(feature = "unstable", feature(slice_ptr_get))]
#![cfg_attr(feature = "very_unstable", feature(const_trait_impl))]
#![cfg_attr(feature = "very_unstable", feature(unboxed_closures))]
#![cfg_attr(feature = "very_unstable", feature(fn_traits))]
#![warn(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub use externally_shared_ptr::ExternallySharedPtr;
pub use externally_shared_ref::ExternallySharedRef;

pub mod access;
mod externally_shared_ptr;
mod externally_shared_ref;
