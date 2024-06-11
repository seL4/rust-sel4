//
// Copyright 2023, Colias Group, LLC
// Copyright 2023, Galois, Inc.
//
// SPDX-License-Identifier: BSD-2-Clause
//

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    PutChar { val: u8 },
    GetChar,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetCharSomeResponse {
    pub val: Option<u8>,
}
