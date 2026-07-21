//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::arch::naked_asm;

use super::main;

mod stack {
    use core::cell::UnsafeCell;

    #[repr(C, align(16))]
    struct Stack<const N: usize>(UnsafeCell<[u8; N]>);

    unsafe impl<const N: usize> Sync for Stack<N> {}

    impl<const N: usize> Stack<N> {
        const fn new() -> Self {
            Self(UnsafeCell::new([0; N]))
        }

        const fn bottom(&self) -> StackBottom {
            StackBottom(self.0.get().cast::<u8>().wrapping_add(N))
        }
    }

    #[repr(transparent)]
    pub(super) struct StackBottom(#[allow(dead_code)] *mut u8);

    unsafe impl Sync for StackBottom {}

    const STACK_SIZE: usize = 0x4000;

    static STACK: Stack<STACK_SIZE> = Stack::new();

    pub(super) static STACK_BOTTOM: StackBottom = STACK.bottom();
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
unsafe extern "C" fn _start() -> ! {
    naked_asm! {
        cfg_select! {
            target_arch = "riscv64" => r#"
                .macro lx dst, src
                    ld \dst, \src
                .endm
            "#,
            target_arch = "riscv32" => r#"
                .macro lx dst, src
                    lw \dst, \src
                .endm
            "#,
            _ => "",
        },
        cfg_select! {
            target_arch = "aarch64" => r#"
                    ldr x9, ={stack_bottom}
                    ldr x9, [x9]
                    mov sp, x9
                    b {rust_entrypoint}
                1:  b 1b
            "#,
            target_arch = "arm" => r#"
                    ldr r12, ={stack_bottom}
                    ldr r12, [r12]
                    mov sp, r12
                    b {rust_entrypoint}
                1:  b 1b
            "#,
            any(target_arch = "riscv64", target_arch = "riscv32") => r#"
                .extern __global_pointer$
                .option push
                .option norelax
                1:  auipc gp, %pcrel_hi(__global_pointer$)
                    addi gp, gp, %pcrel_lo(1b)
                .option pop

                    la sp, {stack_bottom}
                    lx sp, (sp)
                    jal {rust_entrypoint}
                1:  j 1b

                .purgem lx
            "#,
            target_arch = "x86_64" => r#"
                    mov rsp, {stack_bottom}
                    mov rbp, rsp
                    sub rsp, 0x8 // Stack must be 16-byte aligned before call
                    push rbp
                    call {rust_entrypoint}
                1:  jmp 1b
            "#,
        },
        stack_bottom = sym stack::STACK_BOTTOM,
        rust_entrypoint = sym rust_entrypoint,
    }
}

unsafe extern "C" fn rust_entrypoint(bootinfo: *const sel4::BootInfo) -> ! {
    let bootinfo = unsafe { sel4::BootInfoPtr::new(bootinfo) };
    match main(&bootinfo) {
        #[allow(unreachable_patterns)]
        Ok(absurdity) => match absurdity {},
        Err(err) => panic!("Error: {}", err),
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    sel4::debug_println!("{}", info);
    loop {}
}
