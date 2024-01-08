//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

// TODO
// - guard pages for stack?

use core::arch::global_asm;
use core::cell::UnsafeCell;

// TODO alignment should depend on configuration
#[repr(C, align(16))]
pub struct Stack<const N: usize>(UnsafeCell<[u8; N]>);

unsafe impl<const N: usize> Sync for Stack<N> {}

impl<const N: usize> Stack<N> {
    pub const fn new() -> Self {
        Self(UnsafeCell::new([0; N]))
    }

    pub const fn top(&self) -> StackTop {
        StackTop(self.0.get().cast::<u8>().wrapping_add(N))
    }
}

#[repr(transparent)]
pub struct StackTop(#[allow(dead_code)] *mut u8);

unsafe impl Sync for StackTop {}

#[macro_export]
macro_rules! declare_stack {
    ($size:expr) => {
        #[no_mangle]
        static __sel4_runtime_stack_top: $crate::_private::start::StackTop = {
            static STACK: $crate::_private::start::Stack<{ $size }> =
                $crate::_private::start::Stack::new();
            unsafe { STACK.top() }
        };
    };
}

cfg_if::cfg_if! {
    if #[cfg(target_arch = "aarch64")] {
        global_asm! {
            r#"
                .extern sel4_runtime_rust_entry
                .extern __sel4_runtime_stack_top

                .section .text

                .global _start
                _start:
                    ldr x9, =__sel4_runtime_stack_top
                    ldr x9, [x9]
                    mov sp, x9
                    b sel4_runtime_rust_entry

                1:  b 1b
            "#
        }
    } else if #[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))] {
        macro_rules! riscv_common {
            () => {
                r#"
                    .extern sel4_runtime_rust_entry
                    .extern __sel4_runtime_stack_top

                    .section .text

                    .global _start
                    _start:

                        # See https://www.sifive.com/blog/all-aboard-part-3-linker-relaxation-in-riscv-toolchain
                    .option push
                    .option norelax
                    1:  auipc gp, %pcrel_hi(__global_pointer$)
                        addi gp, gp, %pcrel_lo(1b)
                    .option pop

                        la sp, __sel4_runtime_stack_top
                        lx sp, (sp)
                        jal sel4_runtime_rust_entry

                    1:  j 1b
                "#
            }
        }

        #[cfg(target_arch = "riscv64")]
        global_asm! {
            r#"
                .macro lx dst, src
                    ld \dst, \src
                .endm
            "#,
            riscv_common!()
        }

        #[cfg(target_arch = "riscv32")]
        global_asm! {
            r#"
                .macro lx dst, src
                    lw \dst, \src
                .endm
            "#,
            riscv_common!()
        }
    } else if #[cfg(target_arch = "x86_64")] {
        global_asm! {
            r#"
                .extern sel4_runtime_rust_entry
                .extern __sel4_runtime_stack_top

                .section .text

                .global _start
                _start:
                    mov rsp, __sel4_runtime_stack_top
                    mov rbp, rsp
                    sub rsp, 0x8 // Stack must be 16-byte aligned before call
                    push rbp
                    call sel4_runtime_rust_entry

                1:  jmp 1b
            "#
        }
    } else {
        compile_error!("unsupported architecture");
    }
}

pub mod _private {
    pub use super::{Stack, StackTop};
}
