//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

/// Forces a static into `.rodata` without causing `.rodata` to end up in a `PF_W` segment
#[macro_export]
macro_rules! rodata_static {
    ($ident:ident: $ty:ty) => {{
        mod asm {
            unsafe extern "C" {
                pub(super) static $ident: $ty;
            }
            core::arch::global_asm! {
                r#"
                    .section .rodata, "a", %progbits
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
    pub use core::mem::{size_of, align_of};
}
