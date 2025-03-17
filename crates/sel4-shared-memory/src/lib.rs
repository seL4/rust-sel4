//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//

#![no_std]
#![cfg_attr(feature = "atomics", feature(core_intrinsics))]
#![cfg_attr(feature = "atomics", allow(internal_features))]

use sel4_abstract_ptr::{access::ReadWrite, AbstractPtr, AbstractRef};

pub use sel4_abstract_ptr::{access, map_field};

mod shared_memory_type;

pub use shared_memory_type::SharedMemory;

pub type SharedMemoryRef<'a, T, A = ReadWrite> = AbstractRef<'a, SharedMemory, T, A>;
pub type SharedMemoryPtr<'a, T, A = ReadWrite> = AbstractPtr<'a, SharedMemory, T, A>;
