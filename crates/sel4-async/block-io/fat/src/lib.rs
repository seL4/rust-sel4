//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

pub use embedded_fat::*;

mod block_io_wrapper;
mod dummy_time_source;

pub use block_io_wrapper::BlockIOWrapper;
pub use dummy_time_source::DummyTimeSource;
