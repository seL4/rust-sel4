//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct MacAddress(pub [u8; 6]);

pub trait GetMacAddress {
    fn get_mac_address(&mut self) -> MacAddress;
}
