//
// Copyright 2023, Colias Group, LLC
// Copyright 2023, Galois, Inc.
//
// SPDX-License-Identifier: BSD-2-Clause
//

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum Request {
    PutChar { val: u8 },
    GetChar,
}

pub(crate) type Response = Result<SuccessResponse, ErrorResponse>;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum SuccessResponse {
    PutChar,
    GetChar { val: Option<u8> },
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ErrorResponse {
    WriteError,
}
