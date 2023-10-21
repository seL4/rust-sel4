//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::fmt;

use sel4_bitfield_ops::Bitfield;

pub(crate) type SeL4Bitfield<T, const N: usize> = Bitfield<[T; N], T>;

include!(concat!(env!("OUT_DIR"), "/types.rs"));
include!(concat!(env!("OUT_DIR"), "/shared_types.rs"));
