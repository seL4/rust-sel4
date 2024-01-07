//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use serde::{Deserialize, Serialize};

pub type Seconds = u32;

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    Now,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NowResponse {
    pub unix_time: Seconds,
}
