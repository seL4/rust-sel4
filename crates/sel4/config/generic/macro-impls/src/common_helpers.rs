//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

macro_rules! parse_or_return {
    ($tokenstream:ident as $ty:ty) => {
        match parse2::<$ty>($tokenstream) {
            Ok(parsed) => parsed,
            Err(err) => {
                return err.to_compile_error();
            }
        }
    };
}

pub(crate) use parse_or_return;
