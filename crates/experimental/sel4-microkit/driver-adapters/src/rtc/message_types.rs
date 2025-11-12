//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use rtcc::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum Request {
    DateTime,
    SetDateTime(NaiveDateTime),
}

pub(crate) type Response = Result<SuccessResponse, ErrorResponse>;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum SuccessResponse {
    DateTime(NaiveDateTime),
    SetDateTime,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ErrorResponse {
    DateTimeError,
    SetDateTimeError,
}
