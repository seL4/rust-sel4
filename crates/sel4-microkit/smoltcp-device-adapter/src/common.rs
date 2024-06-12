//
// Copyright 2023, Colias Group, LLC
// Copyright 2023, Galois, Inc.
//
// SPDX-License-Identifier: BSD-2-Clause
//

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MacAddress(pub [u8; 6]);

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum Request {
    GetMacAddress,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct GetMacAddressResponse {
    pub mac_address: MacAddress,
}
