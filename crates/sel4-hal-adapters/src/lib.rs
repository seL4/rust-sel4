//
// Copyright 2023, Colias Group, LLC
// Copyright 2023, Galois, Inc.
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(never_type)]
#![feature(strict_provenance)]
#![feature(let_chains)]

#[cfg(feature = "smoltcp-hal")]
pub mod smoltcp;
