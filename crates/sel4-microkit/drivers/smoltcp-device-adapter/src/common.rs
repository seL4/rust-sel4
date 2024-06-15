//
// Copyright 2023, Colias Group, LLC
// Copyright 2023, Galois, Inc.
//
// SPDX-License-Identifier: BSD-2-Clause
//

use serde::{Deserialize, Serialize};

use sel4_driver_traits::MacAddress;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum Request {
    GetMacAddress,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct GetMacAddressResponse {
    pub mac_address: MacAddress,
}
