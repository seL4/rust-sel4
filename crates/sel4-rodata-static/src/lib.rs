//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

/// Forces a static into `.rodata` without causing `.rodata` to end up in a `PF_W` segment
#[macro_export]
macro_rules! rodata_static {
    ($ident:ident: $ty:ty) => {{
        mod asm {
            use super::*;
            unsafe extern "C" {
                pub(super) static $ident: $ty;
            }
            $crate::_private::global_asm! {
                r#"
                    .section .rodata.rodata_static.{ident}, "aR", %progbits
                    .global {ident}
                    .size {ident}, {size}
                    .p2align {align}
                    {ident}:
                        .skip {size}, 0
                "#,
                ident = sym $ident,
                size = const $crate::_private::size_of::<$ty>(),
                align = const $crate::_private::align_of::<$ty>(),
            }
        }
        unsafe { &asm::$ident }
    }};
}

#[doc(hidden)]
pub mod _private {
    pub use core::arch::global_asm;
    pub use core::mem::{align_of, size_of};
}
