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

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PutCharResponse;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct GetCharResponse {
    pub val: Option<u8>,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct PutCharError;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct GetCharError;
