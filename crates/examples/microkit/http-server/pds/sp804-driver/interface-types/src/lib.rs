//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use serde::{Deserialize, Serialize};

pub type Microseconds = u64;

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    Now,
    SetTimeout { relative_micros: Microseconds },
    ClearTimeout,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NowResponse {
    pub micros: Microseconds,
}
