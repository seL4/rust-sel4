//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::arch::global_asm;

#[macro_export]
macro_rules! declare_stack {
    ($size:expr) => {
        const _: () = {
            #[allow(non_upper_case_globals)]
            #[unsafe(no_mangle)]
            static __sel4_runtime_common__stack_bottom: $crate::_private::StackBottom = {
                static STACK: $crate::_private::Stack<{ $size }> = $crate::_private::Stack::new();
                STACK.bottom()
            };
        };
    };
}

#[macro_export]
macro_rules! declare_entrypoint_with_stack_init {
    ($f:ident($( $i:ident: $t:ty ),* $(,)?)) => {
        const _: () = {
            #[unsafe(no_mangle)]
            unsafe extern "C" fn __sel4_runtime_common__rust_entrypoint($($i: $t,)*) -> ! {
                $crate::_private::_run_entrypoint(true, || {
                    $f($($i,)*)
                });
            }
        };

        $crate::_private::global_asm! {
            r#"
                .extern __sel4_runtime_common__stack_init

                .section .text

                .global _start
                _start:
            "#,
            #[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
            r#"
                    b __sel4_runtime_common__stack_init
            "#,
            #[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))]
            r#"
                    j __sel4_runtime_common__stack_init
            "#,
            #[cfg(target_arch = "x86_64")]
            r#"
                    jmp __sel4_runtime_common__stack_init
            "#,
        }
    };
}

global_asm! {
    r#"
        .extern __sel4_runtime_common__stack_bottom
        .extern __sel4_runtime_common__call_rust_entrypoint

        .section .text.__sel4_runtime_common__stack_init, "ax", %progbits

        .global __sel4_runtime_common__stack_init
        __sel4_runtime_common__stack_init:
    "#,
    #[cfg(target_arch = "aarch64")]
    r#"
            ldr x9, =__sel4_runtime_common__stack_bottom
            ldr x9, [x9]
            mov sp, x9
            b __sel4_runtime_common__rust_entrypoint
    "#,
    #[cfg(target_arch = "arm")]
    r#"
            ldr r8, =__sel4_runtime_common__stack_bottom
            ldr r8, [r8]
            mov sp, r8
            b __sel4_runtime_common__rust_entrypoint
    "#,
    #[cfg(target_arch = "riscv64")]
    r#"
            la sp, __sel4_runtime_common__stack_bottom
            ld sp, (sp)
            j __sel4_runtime_common__rust_entrypoint
    "#,
    #[cfg(target_arch = "riscv32")]
    r#"
            la sp, __sel4_runtime_common__stack_bottom
            lw sp, (sp)
            j __sel4_runtime_common__rust_entrypoint
    "#,
    #[cfg(target_arch = "x86_64")]
    r#"
            mov rsp, __sel4_runtime_common__stack_bottom
            mov rbp, rsp
            sub rsp, 0x8 // Stack must be 16-byte aligned before call
            push rbp
            call __sel4_runtime_common__rust_entrypoint
        1:  jmp 1b
    "#,
}
