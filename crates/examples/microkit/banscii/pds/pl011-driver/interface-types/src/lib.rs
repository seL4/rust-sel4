//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    PutChar { val: u8 },
    GetChar,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetCharResponse {
    pub val: Option<u8>,
}
