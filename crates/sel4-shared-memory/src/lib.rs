//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//

#![no_std]
#![cfg_attr(feature = "atomics", feature(cfg_target_has_atomic_equal_alignment))]
#![cfg_attr(feature = "atomics", feature(core_intrinsics))]
#![cfg_attr(feature = "atomics", allow(internal_features))]

use sel4_abstract_ptr::{access::ReadWrite, memory_type::MemoryType, AbstractPtr, AbstractRef};

pub use sel4_abstract_ptr::{access, map_field};

mod ops;

#[cfg(feature = "atomics")]
mod atomic_ops;

#[cfg(feature = "atomics")]
pub use atomic_ops::Atomic;

pub struct SharedMemory(());

impl MemoryType for SharedMemory {}

pub type SharedMemoryRef<'a, T, A = ReadWrite> = AbstractRef<'a, SharedMemory, T, A>;
pub type SharedMemoryPtr<'a, T, A = ReadWrite> = AbstractPtr<'a, SharedMemory, T, A>;
