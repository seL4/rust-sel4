//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

// HACK to force variables into .rodata without causing .rodata to end up in a PF_W segment
macro_rules! rodata_var {
    ($ident:ident: $ty:ty) => {{
        mod asm {
            unsafe extern "C" {
                pub(super) static $ident: $ty;
            }
            core::arch::global_asm! {
                r#"
                    .section .rodata
                    .global {ident}
                    .size {ident}, {size}
                    .p2align {align}
                    {ident}:
                        .skip {size}, 0
                "#,
                ident = sym $ident,
                size = const core::mem::size_of::<$ty>(),
                align = const core::mem::align_of::<$ty>(),
            }
        }
        unsafe { &asm::$ident }
    }};
}

pub(crate) use rodata_var;
