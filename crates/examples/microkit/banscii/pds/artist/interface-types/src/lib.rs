//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub height: usize,
    pub width: usize,
    pub draft_start: usize,
    pub draft_size: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub height: usize,
    pub width: usize,
    pub masterpiece_start: usize,
    pub masterpiece_size: usize,
    pub signature_start: usize,
    pub signature_size: usize,
}
