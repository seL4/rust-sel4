//
// Copyright 2026, Colias Group, LLC
// Copyright 2020, Data61, CSIRO (ABN 41 687 119 230)
// Copyright (C) 2012 ARM Ltd.
// Copyright (C) 1999-2002 Russell King.
//
// SPDX-License-Identifier: GPL-2.0-only
//

use core::arch::naked_asm;

// See:
// https://developer.arm.com/documentation/den0024/a/Caches/Cache-maintenance
macro_rules! dcache_op {
    ($op:literal, $self:path) => {
        naked_asm! {
            concat!(
                r#"
                        dsb     sy                          // ensure ordering with previous memory accesses

                        mrs     x0, clidr_el1               // extract LoC << 1 from CLIDR
                        and     x3, x0, #0x7000000
                        lsr     x3, x3, #23

                        cbz     x3, .L{self}_finished       // if loc is 0, then no need to clean

                        mov     x10, #0                     // start clean at cache level 0

                    1:  add     x2, x10, x10, lsr #1
                        lsr     x1, x0, x2
                        and     x1, x1, #7
                        cmp     x1, #2
                        b.lt    .L{self}_skip

                        msr     csselr_el1, x10
                        isb

                        mrs     x1, ccsidr_el1
                        and     x2, x1, #7
                        add     x2, x2, #4
                        mov     x4, #0x3ff
                        and     x4, x4, x1, lsr #3
                        clz     w5, w4
                        mov     x7, #0x7fff
                        and     x7, x7, x1, lsr #13

                    2:  mov     x9, x4

                    3:  lsl     x6, x9, x5
                        orr     x11, x10, x6
                        lsl     x6, x7, x2
                        orr     x11, x11, x6
                        dc      "#, $op, r#", x11
                        subs    x9, x9, #1
                        b.ge    3b
                        subs    x7, x7, #1
                        b.ge    2b

                    .L{self}_skip:
                        add     x10, x10, #2
                        cmp     x3, x10
                        b.gt    1b

                    .L{self}_finished:
                        mov     x10, #0
                        msr     csselr_el1, x10
                        dsb     sy
                        isb
                        ret
                "#,
            ),
            self = sym $self,
        }
    };
}

#[unsafe(no_mangle)]
#[unsafe(naked)]
pub(crate) extern "C" fn invalidate_dcache() {
    dcache_op!("isw", invalidate_dcache);
}

#[unsafe(no_mangle)]
#[unsafe(naked)]
pub(crate) extern "C" fn clean_and_invalidate_dcache() {
    dcache_op!("cisw", clean_and_invalidate_dcache);
}

#[unsafe(no_mangle)]
#[unsafe(naked)]
pub(crate) extern "C" fn invalidate_icache() {
    naked_asm! {
        r#"
            ic      iallu
            dsb     nsh
            isb
            ret
        "#
    }
}
