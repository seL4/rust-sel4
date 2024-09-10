//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]
#![feature(core_intrinsics)]
#![allow(internal_features)]

use core::arch::global_asm;

fn main(bootinfo: &sel4::BootInfoPtr) -> sel4::Result<Never> {
    sel4::debug_println!("Hello, World!");

    let mut ipc_buffer = unsafe { bootinfo.ipc_buffer().as_mut().unwrap() };

    let blueprint = sel4::ObjectBlueprint::Notification;

    let chosen_untyped_ix = bootinfo
        .untyped_list()
        .iter()
        .position(|desc| !desc.is_device() && desc.size_bits() >= blueprint.physical_size_bits())
        .unwrap();

    let untyped = bootinfo.untyped().index(chosen_untyped_ix).cap();

    let mut empty_slots = bootinfo
        .empty()
        .range()
        .map(sel4::init_thread::Slot::from_index);
    let unbadged_notification_slot = empty_slots.next().unwrap();
    let badged_notification_slot = empty_slots.next().unwrap();

    let cnode = sel4::init_thread::slot::CNODE.cap();

    untyped.with(&mut ipc_buffer).untyped_retype(
        &blueprint,
        &cnode.relative_self(),
        unbadged_notification_slot.index(),
        1,
    )?;

    let badge = 0x1337;

    cnode
        .with(&mut ipc_buffer)
        .relative(badged_notification_slot.cptr())
        .mint(
            &cnode.relative(unbadged_notification_slot.cptr()),
            sel4::CapRights::write_only(),
            badge,
        )?;

    let unbadged_notification = unbadged_notification_slot
        .downcast::<sel4::cap_type::Notification>()
        .cap();
    let badged_notification = badged_notification_slot
        .downcast::<sel4::cap_type::Notification>()
        .cap();

    badged_notification.with(&mut ipc_buffer).signal();

    let (_, observed_badge) = unbadged_notification.with(&mut ipc_buffer).wait();

    sel4::debug_println!("badge = {:#x}", badge);
    assert_eq!(observed_badge, badge);

    sel4::debug_println!("TEST_PASS");

    sel4::init_thread::slot::TCB
        .cap()
        .with(&mut ipc_buffer)
        .tcb_suspend()
        .unwrap();

    unreachable!()
}

// minimal ad-hoc runtime

enum Never {}

mod stack {
    use core::cell::UnsafeCell;

    #[repr(C, align(16))]
    pub struct Stack<const N: usize>(UnsafeCell<[u8; N]>);

    unsafe impl<const N: usize> Sync for Stack<N> {}

    impl<const N: usize> Stack<N> {
        pub const fn new() -> Self {
            Self(UnsafeCell::new([0; N]))
        }

        pub const fn bottom(&self) -> StackBottom {
            StackBottom(self.0.get().cast::<u8>().wrapping_add(N))
        }
    }

    #[repr(transparent)]
    pub struct StackBottom(#[allow(dead_code)] *mut u8);

    unsafe impl Sync for StackBottom {}

    const STACK_SIZE: usize = 0x4000;

    static STACK: Stack<STACK_SIZE> = Stack::new();

    #[no_mangle]
    static __stack_bottom: StackBottom = STACK.bottom();
}

cfg_if::cfg_if! {
    if #[cfg(target_arch = "aarch64")] {
        global_asm! {
            r#"
                .extern __rust_entry
                .extern __stack_bottom

                .section .text

                .global _start
                _start:
                    ldr x9, =__stack_bottom
                    ldr x9, [x9]
                    mov sp, x9
                    b __rust_entry

                1:  b 1b
            "#
        }
    } else if #[cfg(target_arch = "arm")] {
            global_asm! {
                r#"
                    .extern __rust_entry
                    .extern __stack_bottom

                    .section .text

                    .global _start
                    _start:
                        ldr r8, =__stack_bottom
                        ldr r8, [r8]
                        mov sp, r8
                        b __rust_entry

                    1:  b 1b
                "#
            }
    } else if #[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))] {
        macro_rules! riscv_common {
            () => {
                r#"
                    .extern __rust_entry
                    .extern __stack_bottom

                    .section .text

                    .global _start
                    _start:

                        # See https://www.sifive.com/blog/all-aboard-part-3-linker-relaxation-in-riscv-toolchain
                    .option push
                    .option norelax
                    1:  auipc gp, %pcrel_hi(__global_pointer$)
                        addi gp, gp, %pcrel_lo(1b)
                    .option pop

                        la sp, __stack_bottom
                        lx sp, (sp)
                        jal __rust_entry

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
                .extern __rust_entry
                .extern __stack_bottom

                .section .text

                .global _start
                _start:
                    mov rsp, __stack_bottom
                    mov rbp, rsp
                    sub rsp, 0x8 // Stack must be 16-byte aligned before call
                    push rbp
                    call __rust_entry

                1:  jmp 1b
            "#
        }
    } else {
        compile_error!("unsupported architecture");
    }
}

#[no_mangle]
unsafe extern "C" fn __rust_entry(bootinfo: *const sel4::BootInfo) -> ! {
    let bootinfo = sel4::BootInfoPtr::new(bootinfo);
    match main(&bootinfo) {
        #[allow(unreachable_patterns)]
        Ok(absurdity) => match absurdity {},
        Err(err) => panic!("Error: {}", err),
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    sel4::debug_println!("{}", info);
    core::intrinsics::abort()
}
