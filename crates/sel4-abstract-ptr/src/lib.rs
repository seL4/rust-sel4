//
// Copyright 2024, Colias Group, LLC
// Copyright (c) 2020 Philipp Oppermann
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//

#![no_std]
#![allow(clippy::missing_safety_doc)]

mod abstract_ptr;
mod abstract_ref;
mod core_ext;

pub mod access;
pub mod memory_type;

pub use abstract_ptr::AbstractPtr;
pub use abstract_ref::AbstractRef;
