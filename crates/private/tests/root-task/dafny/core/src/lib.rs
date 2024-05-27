//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(proc_macro_hygiene)]

use dafny_runtime::DafnyInt;
use num::traits::cast::ToPrimitive;
use sel4_mod_in_out_dir::in_out_dir;

#[rustfmt::skip]
#[in_out_dir]
mod translated;

pub fn max(a: usize, b: usize) -> usize {
    translated::_module::_default::Max(&DafnyInt::from(a), &DafnyInt::from(b))
        .to_usize()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::max;

    #[test]
    fn x() {
        assert_eq!(max(13, 37), 37);
    }
}
