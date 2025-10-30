//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

// TODO
// - guard pages for stack?

use core::arch::global_asm;

#[macro_export]
macro_rules! declare_stack {
    ($size:expr) => {
        #[allow(non_upper_case_globals)]
        #[unsafe(no_mangle)]
        static __sel4_runtime_common__stack_bottom: $crate::_private::start::StackBottom = {
            static STACK: $crate::_private::start::Stack<{ $size }> =
                $crate::_private::start::Stack::new();
            STACK.bottom()
        };
    };
}

#[macro_export]
macro_rules! declare_entrypoint {
    (($( $i:ident: $t:ty ),* $(,)?) -> ! $body:block) => {
        #[allow(unreachable_code)]
        #[unsafe(no_mangle)]
        unsafe extern "C" fn __sel4_runtime_common__rust_entrypoint($($i: $t,)*) -> ! {
            $crate::_private::start::with_local_initialization(|| {
                $crate::_private::start::global_initialzation();
                $body
            });
            $crate::_private::start::abort!("entrypoint returned")
        }
    };
}

macro_rules! common_asm_prefix {
    () => {
        r#"
            .extern __sel4_runtime_common__rust_entrypoint
            .extern __sel4_runtime_common__stack_bottom

            .global _start

            .section .text

            _start:
        "#
    };
}

cfg_if::cfg_if! {
    if #[cfg(target_arch = "aarch64")] {
        global_asm! {
            common_asm_prefix!(),
            r#"
                    ldr x9, =__sel4_runtime_common__stack_bottom
                    ldr x9, [x9]
                    mov sp, x9
                    b __sel4_runtime_common__rust_entrypoint

                1:  b 1b
            "#
        }
    } else if #[cfg(target_arch = "arm")] {
        global_asm! {
            common_asm_prefix!(),
            r#"
                    ldr r8, =__sel4_runtime_common__stack_bottom
                    ldr r8, [r8]
                    mov sp, r8
                    b __sel4_runtime_common__rust_entrypoint

                1:  b 1b
            "#
        }
    } else if #[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))] {
        macro_rules! riscv_common_asm_body {
            () => {
                r#"
                        # See https://www.sifive.com/blog/all-aboard-part-3-linker-relaxation-in-riscv-toolchain
                    .option push
                    .option norelax
                    1:  auipc gp, %pcrel_hi(__global_pointer$)
                        addi gp, gp, %pcrel_lo(1b)
                    .option pop

                        la sp, __sel4_runtime_common__stack_bottom
                        __sel4_runtime_common__lx sp, (sp)
                        jal __sel4_runtime_common__rust_entrypoint

                    2:  j 2b
                "#
            }
        }

        #[cfg(target_arch = "riscv64")]
        global_asm! {
            r#"
                .macro __sel4_runtime_common__lx dst, src
                    ld \dst, \src
                .endm
            "#,
            common_asm_prefix!(),
            riscv_common_asm_body!()
        }

        #[cfg(target_arch = "riscv32")]
        global_asm! {
            r#"
                .macro __sel4_runtime_common__lx dst, src
                    lw \dst, \src
                .endm
            "#,
            common_asm_prefix!(),
            riscv_common_asm_body!()
        }
    } else if #[cfg(target_arch = "x86_64")] {
        global_asm! {
            common_asm_prefix!(),
            r#"
                    mov rsp, __sel4_runtime_common__stack_bottom
                    mov rbp, rsp
                    sub rsp, 0x8 // Stack must be 16-byte aligned before call
                    push rbp
                    call __sel4_runtime_common__rust_entrypoint

                1:  jmp 1b
            "#
        }
    } else {
        compile_error!("unsupported architecture");
    }
}

pub mod _private {
    pub use sel4_panicking_env::abort;
    pub use sel4_stack::{Stack, StackBottom};

    pub use crate::{global_initialzation, with_local_initialization};
}
