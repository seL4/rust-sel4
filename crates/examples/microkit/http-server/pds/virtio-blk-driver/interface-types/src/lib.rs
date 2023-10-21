//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    GetNumBlocks,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetNumBlocksResponse {
    pub num_blocks: u64,
}
