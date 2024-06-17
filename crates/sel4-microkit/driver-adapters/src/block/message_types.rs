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
    GetBlockSize,
    GetNumBlocks,
}

pub(crate) type Response = Result<SuccessResponse, ErrorResponse>;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum SuccessResponse {
    GetBlockSize(usize),
    GetNumBlocks(u64),
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ErrorResponse {}
