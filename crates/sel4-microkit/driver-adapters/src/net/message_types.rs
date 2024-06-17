//
// Copyright 2023, Colias Group, LLC
// Copyright 2023, Galois, Inc.
//
// SPDX-License-Identifier: BSD-2-Clause
//

use serde::{Deserialize, Serialize};

use sel4_driver_interfaces::net::MacAddress;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum Request {
    GetMacAddress,
}

pub(crate) type Response = Result<SuccessResponse, ErrorResponse>;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum SuccessResponse {
    GetMacAddress { mac_address: MacAddress },
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ErrorResponse {}
