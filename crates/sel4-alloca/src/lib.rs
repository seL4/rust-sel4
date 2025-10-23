//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::alloc::Layout;
use core::arch::global_asm;
use core::mem::{ManuallyDrop, MaybeUninit};
use core::slice;

use cfg_if::cfg_if;

// TODO:
// - support unwinding

pub fn with_alloca_bytes<R, F: FnOnce(&mut [MaybeUninit<u8>]) -> R>(layout: Layout, f: F) -> R {
    with_alloca_ptr(layout, |p| {
        f(unsafe { slice::from_raw_parts_mut(p.cast(), layout.size()) })
    })
}

pub fn with_alloca<R, T, F: FnOnce(&mut MaybeUninit<T>) -> R>(f: F) -> R {
    with_alloca_ptr(Layout::new::<T>(), |p| f(unsafe { &mut *p.cast() }))
}

pub fn with_alloca_slice<R, T, F: FnOnce(&mut [MaybeUninit<T>]) -> R>(n: usize, f: F) -> R {
    with_alloca_ptr(Layout::array::<T>(n).unwrap(), |p| {
        f(unsafe { slice::from_raw_parts_mut(p.cast(), n) })
    })
}

pub fn with_alloca_ptr<R, F: FnOnce(*mut u8) -> R>(layout: Layout, f: F) -> R {
    unsafe extern "C" fn cont_fn<F: FnOnce(*mut u8)>(
        reservation_start: *mut u8,
        cont_arg: *mut ReserveOnStackContArg,
    ) {
        let f = ManuallyDrop::take(&mut *(cont_arg as *mut ManuallyDrop<F>));
        f(reservation_start)
    }

    #[inline(always)]
    fn get_cont_fn<F: FnOnce(*mut u8)>(_closure: &F) -> ReserveOnStackContFn {
        cont_fn::<F>
    }

    let mut ret = MaybeUninit::uninit();

    let closure = |p| {
        ret.write(f(p));
    };

    let inst_cont_fn = get_cont_fn(&closure);
    let mut closure_data = ManuallyDrop::new(closure);

    unsafe {
        reserve_on_stack(
            layout,
            inst_cont_fn,
            &mut closure_data as *mut _ as *mut ReserveOnStackContArg,
        );
        ret.assume_init()
    }
}

type ReserveOnStackContFn =
    unsafe extern "C" fn(reservation_start: *mut u8, cont_arg: *mut ReserveOnStackContArg);

enum ReserveOnStackContArg {}

#[allow(clippy::missing_safety_doc)]
unsafe fn reserve_on_stack(
    layout: Layout,
    cont_fn: ReserveOnStackContFn,
    cont_arg: *mut ReserveOnStackContArg,
) {
    let reservation_size = layout.size();
    let reservation_align_down_mask = !(layout.align() - 1);
    unsafe {
        __sel4_alloca__reserve_on_stack(
            reservation_size,
            reservation_align_down_mask,
            cont_fn,
            cont_arg,
        )
    }
}

unsafe extern "C" {
    fn __sel4_alloca__reserve_on_stack(
        reservation_size: usize,
        reservation_align_down_mask: usize,
        cont_fn: ReserveOnStackContFn,
        cont_arg: *mut ReserveOnStackContArg,
    );
}

macro_rules! common_asm_prefix {
    () => {
        r#"
            .section .text
            .global __sel4_alloca__reserve_on_stack
            __sel4_alloca__reserve_on_stack:
        "#
    };
}

cfg_if! {
    if #[cfg(target_arch = "aarch64")] {
        global_asm! {
            common_asm_prefix!(),
            r#"
                // preamble
                stp fp, lr, [sp, #-16]!
                mov fp, sp

                mov x9, sp
                sub x9, x9, x0        // x0: reservation_size
                and x9, x9, x1        // x1: reservation_align_down_mask
                mov x10, x9           // save reservation_start for later
                and x9, x9, ~(16 - 1) // align stack
                mov sp, x9

                mov x0, x10           // pass reservation_start
                mov x1, x3            // pass cont_arg
                blr x2                // call cont_fn

                // postamble
                mov sp, fp
                ldp fp, lr, [sp], #16
                ret
            "#
        }
    } else if #[cfg(target_arch = "arm")] {
        global_asm! {
            common_asm_prefix!(),
            r#"
                // preamble
                push {{r11, lr}}
                add r11, sp, #0

                mov r4, sp
                sub r4, r4, r0        // r0: reservation_size
                and r4, r4, r1        // r1: reservation_align_down_mask
                mov r6, r4            // save reservation_start for later
                and r4, r4, ~(4 - 1)  // align stack
                mov sp, r4

                mov r0, r6            // pass reservation_start
                mov r1, r3            // pass cont_arg
                blx r2                // call cont_fn

                // postamble
                sub sp, r11, #0
                pop {{r11, pc}}
            "#
        }
    } else if #[cfg(target_arch = "riscv64")] {
        global_asm! {
            common_asm_prefix!(),
            r#"
                // preamble
                sd ra, -8(sp)
                sd s0, -16(sp)
                mv s0, sp
                addi sp, sp, -16

                mv t0, sp
                sub t0, t0, a0        // a0: reservation_size
                and t0, t0, a1        // a1: reservation_align_down_mask
                mv t1, t0             // save reservation_start for later
                and t0, t0, ~(16 - 1) // align stack
                mv sp, t0

                mv a0, t1             // pass reservation_start
                mv a1, a3             // pass cont_arg
                jalr a2               // call cont_fn

                // postamble
                mv sp, s0
                ld ra, -8(sp)
                ld s0, -16(sp)
                ret
            "#
        }
    } else if #[cfg(target_arch = "riscv32")] {
        global_asm! {
            common_asm_prefix!(),
            r#"
                // preamble
                sw ra, -4(sp)
                sw s0, -8(sp)
                mv s0, sp
                addi sp, sp, -16

                mv t0, sp
                sub t0, t0, a0        // a0: reservation_size
                and t0, t0, a1        // a1: reservation_align_down_mask
                mv t1, t0             // save reservation_start for later
                and t0, t0, ~(16 - 1) // align stack
                mv sp, t0

                mv a0, t1             // pass reservation_start
                mv a1, a3             // pass cont_arg
                jalr a2               // call cont_fn

                // postamble
                mv sp, s0
                lw ra, -4(sp)
                lw s0, -8(sp)
                ret
            "#
        }
    } else if #[cfg(target_arch = "x86_64")] {
        global_asm! {
            common_asm_prefix!(),
            r#"
                // preamble
                push rbp
                mov rbp, rsp

                mov r10, rsp
                sub r10, rdi          // rdi: reservation_size
                and r10, rsi          // rsi: reservation_align_down_mask
                mov rax, r10          // save reservation_start for later
                and r10, ~(16 - 1)    // align stack
                mov rsp, r10

                mov rdi, rax          // pass reservation_start
                mov rsi, rcx          // pass cont_arg
                call rdx              // call cont_fn

                // postamble
                leave
                ret
            "#
        }
    } else {
        compile_error!("unsupported architecture");
    }
}
