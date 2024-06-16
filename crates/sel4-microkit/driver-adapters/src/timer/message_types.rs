//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum Request {
    GetTime,
    NumTimers,
    SetTimeout { timer: usize, relative: Duration },
    ClearTimeout { timer: usize },
}

pub(crate) type Response = Result<SuccessResponse, ErrorResponse>;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum SuccessResponse {
    GetTime(Duration),
    NumTimers(usize),
    SetTimeout,
    ClearTimeout,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ErrorResponse {
    TimerOutOfBounds,
    Unspecified,
}
