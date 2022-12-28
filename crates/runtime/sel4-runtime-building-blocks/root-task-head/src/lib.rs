#![no_std]
#![no_main]
#![feature(core_intrinsics)]
#![feature(exclusive_wrapper)]

use core::arch::global_asm;
use core::sync::Exclusive;

const STACK_SIZE: usize = include!(concat!(env!("OUT_DIR"), "/stack_size.fragment.rs"));

#[repr(C, align(16))]
struct Stack([u8; STACK_SIZE]);

static mut STACK: Stack = Stack([0; STACK_SIZE]);

#[no_mangle]
static __stack_top: Exclusive<*const u8> = Exclusive::new(unsafe { STACK.0.as_ptr_range().end });

cfg_if::cfg_if! {
    if #[cfg(target_arch = "aarch64")] {
        global_asm! {
            r#"
                .extern __rust_entry
                .extern __stack_top

                .section .text

                .global _start
                _start:
                    ldr x9, =__stack_top
                    ldr x9, [x9]
                    mov sp, x9
                    b __rust_entry
            "#
        }
    } else if #[cfg(target_arch = "x86_64")] {
        global_asm! {
            r#"
                .extern __rust_entry
                .extern __stack_top

                .section .text

                .global _start
                _start:
                    mov rsp, __stack_top
                    mov rbp, rsp
                    sub rsp, 0x8 // Stack must be 16-byte aligned before call
                    push rbp
                    call __rust_entry
            "#
        }
    } else {
        compile_error!("unsupported architecture");
    }
}
