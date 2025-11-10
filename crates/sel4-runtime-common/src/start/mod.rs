//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::arch::global_asm;

use cfg_if::cfg_if;

mod with_stack_init;

#[macro_export]
macro_rules! declare_entrypoint {
    {
        $f:ident($( $i:ident: $t:ty ),* $(,)?)
     } => {
        $crate::declare_entrypoint! {
            $f($($i: $t,)*)
            global_init if true
        }
    };
    {
        $f:ident($( $i:ident: $t:ty ),* $(,)?)
        global_init if $global_init_cond:expr
     } => {
        const _: () = {
            #[unsafe(no_mangle)]
            unsafe extern "C" fn __sel4_runtime_common__rust_entrypoint($($i: $t,)*) -> ! {
                $crate::_private::_run_entrypoint($global_init_cond, || {
                    $f($($i,)*)
                });
            }
        };

        // TODO make this much prettier once #[feature(asm_cfg)] stabilizes
        $crate::_private::cfg_if! {
            if #[cfg(any(target_arch = "aarch64", target_arch = "arm"))] {
                $crate::_private::global_asm! {
                    r#"
                        .extern __sel4_runtime_common__call_rust_entrypoint

                        .section .text

                        .global _start
                        _start:
                            b __sel4_runtime_common__call_rust_entrypoint
                    "#
                }
            } else if #[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))] {
                $crate::_private::global_asm! {
                    r#"
                        .extern __sel4_runtime_common__call_rust_entrypoint

                        .section .text

                        .global _start
                        _start:
                            j __sel4_runtime_common__call_rust_entrypoint
                    "#
                }
            } else if #[cfg(target_arch = "x86_64")] {
                $crate::_private::global_asm! {
                    r#"
                        .extern __sel4_runtime_common__call_rust_entrypoint

                        .section .text

                        .global _start
                        _start:
                            jmp __sel4_runtime_common__call_rust_entrypoint
                    "#
                }
            } else {
                compile_error!("unsupported architecture");
            }
        }
    };
}

macro_rules! common_asm_prefix {
    () => {
        r#"
            .extern __sel4_runtime_common__rust_entrypoint

            .section .text.__sel4_runtime_common__call_rust_entrypoint, "ax", %progbits

            .global __sel4_runtime_common__call_rust_entrypoint
            __sel4_runtime_common__call_rust_entrypoint:
        "#
    };
}

cfg_if! {
    if #[cfg(any(target_arch = "aarch64", target_arch = "arm"))] {
        global_asm! {
            common_asm_prefix!(),
            r#"
                    bl __sel4_runtime_common__rust_entrypoint
                1:  b 1b
            "#
        }
    } else if #[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))] {
        global_asm! {
            common_asm_prefix!(),
            r#"
                .option push
                .option norelax
                1:  auipc gp, %pcrel_hi(__global_pointer$)
                    addi gp, gp, %pcrel_lo(1b)
                .option pop
                    jal __sel4_runtime_common__rust_entrypoint
                1:  j 1b
            "#
        }
    } else if #[cfg(target_arch = "x86_64")] {
        global_asm! {
            common_asm_prefix!(),
            r#"
                    call __sel4_runtime_common__rust_entrypoint
                1:  jmp 1b
            "#
        }
    } else {
        compile_error!("unsupported architecture");
    }
}
