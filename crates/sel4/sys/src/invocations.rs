//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use crate::{
    bf::*,
    c::*,
};

pub mod invocation_label {
    include!(concat!(env!("OUT_DIR"), "/invocation_labels.rs"));
}

include!(concat!(env!("OUT_DIR"), "/invocations.rs"));
