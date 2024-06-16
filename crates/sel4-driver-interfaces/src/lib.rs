//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use serde::{Deserialize, Serialize};

pub trait HandleInterrupt {
    fn handle_interrupt(&mut self);
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct MacAddress(pub [u8; 6]);

pub trait GetMacAddress {
    fn get_mac_address(&mut self) -> MacAddress;
}
