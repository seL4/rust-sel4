//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![allow(clippy::type_complexity)]

mod gen {
    include!(concat!(env!("OUT_DIR"), "/spec.rs"));
}

pub use gen::SPEC;
