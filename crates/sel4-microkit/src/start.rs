//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::arch::global_asm;

macro_rules! common_asm_prefix {
    () => {
        r#"
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
                    b __sel4_microkit__rust_entrypoint
                1:  b 1b
            "#
        }
    } else if #[cfg(target_arch = "arm")] {
        global_asm! {
            common_asm_prefix!(),
            r#"
                    b __sel4_microkit__rust_entrypoint
                1:  b 1b
            "#
        }
    } else if #[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))] {
        global_asm! {
            common_asm_prefix!(),
            r#"
                    # See https://www.sifive.com/blog/all-aboard-part-3-linker-relaxation-in-riscv-toolchain
                .option push
                .option norelax
                1:  auipc gp, %pcrel_hi(__global_pointer$)
                    addi gp, gp, %pcrel_lo(1b)
                .option pop
                    jal __sel4_microkit__rust_entrypoint
                2:  j 2b
            "#
        }
    } else if #[cfg(target_arch = "x86_64")] {
        global_asm! {
            common_asm_prefix!(),
            r#"
                    call __sel4_microkit__rust_entrypoint
                1:  jmp 1b
            "#
        }
    } else {
        compile_error!("unsupported architecture");
    }
}
