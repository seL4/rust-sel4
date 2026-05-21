//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::arch::naked_asm;

use sel4_stack::StackBottom;

#[macro_export]
macro_rules! declare_entrypoint {
    () => {
        #[unsafe(naked)]
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn _start() -> ! {
            $crate::_private::naked_asm! {
                $crate::_private::cfg_select! {
                    any(target_arch = "aarch64", target_arch = "arm") => r#"
                        b {call_rust_entrypoint}
                    "#,
                    any(target_arch = "riscv32", target_arch = "riscv64") => r#"
                        j {call_rust_entrypoint}
                    "#,
                    target_arch = "x86_64" => r#"
                        jmp {call_rust_entrypoint}
                    "#,
                },
                call_rust_entrypoint = sym $crate::_private::call_rust_entrypoint,
            }
        }
    };
}

#[macro_export]
macro_rules! declare_entrypoint_with_stack_init {
    () => {
        #[unsafe(naked)]
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn _start() -> ! {
            $crate::_private::naked_asm! {
                $crate::_private::cfg_select! {
                    any(target_arch = "aarch64", target_arch = "arm") => r#"
                        b {stack_init}
                    "#,
                    any(target_arch = "riscv32", target_arch = "riscv64") => r#"
                        j {stack_init}
                    "#,
                    target_arch = "x86_64" => r#"
                        jmp {stack_init}
                    "#,
                },
                stack_init = sym $crate::_private::stack_init,
            }
        }
    };
}

#[macro_export]
macro_rules! declare_stack {
    ($size:expr) => {
        const _: () = {
            #[unsafe(no_mangle)]
            static __sel4_runtime_common__stack_bottom: $crate::_private::StackBottom = {
                static STACK: $crate::_private::Stack<{ $size }> = $crate::_private::Stack::new();
                STACK.bottom()
            };
        };
    };
}

unsafe extern "C" {
    static __sel4_runtime_common__stack_bottom: StackBottom;
    static __sel4_runtime_common__rust_entrypoint: usize; // until #[feature(abi_custom)]
}

#[doc(hidden)]
#[unsafe(naked)]
pub unsafe extern "C" fn call_rust_entrypoint() -> ! {
    naked_asm! {
        cfg_select! {
            any(target_arch = "aarch64", target_arch = "arm") => r#"
                    bl {rust_entrypoint}
                1:  b 1b
            "#,
            any(target_arch = "riscv32", target_arch = "riscv64") => r#"
                .option push
                .option norelax
                1:  auipc gp, %pcrel_hi(__global_pointer$)
                    addi gp, gp, %pcrel_lo(1b)
                .option pop
                    jal {rust_entrypoint}
                1:  j 1b
            "#,
            target_arch = "x86_64" => r#"
                    call {rust_entrypoint}
                1:  jmp 1b
            "#,
        },
        rust_entrypoint = sym __sel4_runtime_common__rust_entrypoint,
    }
}

#[doc(hidden)]
#[unsafe(naked)]
pub unsafe extern "C" fn stack_init() -> ! {
    naked_asm! {
        cfg_select! {
            target_arch = "aarch64" => r#"
                ldr x9, ={stack_bottom}
                ldr x9, [x9]
                mov sp, x9
                b {call_rust_entrypoint}
            "#,
            target_arch = "arm" => r#"
                ldr r12, ={stack_bottom}
                ldr r12, [r12]
                mov sp, r12
                b {call_rust_entrypoint}
            "#,
            target_arch = "riscv64" => r#"
                la sp, {stack_bottom}
                ld sp, (sp)
                j {call_rust_entrypoint}
            "#,
            target_arch = "riscv32" => r#"
                la sp, {stack_bottom}
                lw sp, (sp)
                j {call_rust_entrypoint}
            "#,
            target_arch = "x86_64" => r#"
                mov rsp, {stack_bottom}
                mov rbp, rsp
                sub rsp, 0x8 // Stack must be 16-byte aligned before call
                push rbp
                call {call_rust_entrypoint}
            "#,
        },
        stack_bottom = sym __sel4_runtime_common__stack_bottom,
        call_rust_entrypoint = sym call_rust_entrypoint,
    }
}
