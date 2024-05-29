//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use vstd::prelude::*;

verus! {
    pub fn max(a: u64, b: u64) -> (ret: u64)
        ensures
            ret == a || ret == b,
            ret >= a && ret >= b,
    {
        if a >= b {
            a
        } else {
            b
        }
    }
}
