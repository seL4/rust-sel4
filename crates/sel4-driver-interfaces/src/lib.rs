//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

pub mod block;
pub mod net;
pub mod timer;

pub trait HandleInterrupt {
    fn handle_interrupt(&mut self);
}
