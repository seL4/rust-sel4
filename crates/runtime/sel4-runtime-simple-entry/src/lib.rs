#![no_std]
#![feature(const_mut_refs)]
#![feature(const_option_ext)]
#![feature(exclusive_wrapper)]

use core::arch::global_asm;
use core::sync::Exclusive;

#[repr(C, align(16))]
pub struct Stack<const N: usize>([u8; N]);

impl<const N: usize> Stack<N> {
    pub const fn new() -> Self {
        Self([0; N])
    }

    pub const fn top(&mut self) -> StackTop {
        StackTop(Exclusive::new(self.0.as_mut_ptr_range().end))
    }
}

#[repr(transparent)]
pub struct StackTop(Exclusive<*mut u8>);

macro_rules! declare_stack {
    ($size:expr) => {
        #[no_mangle]
        static __sel4_runtime_stack_top: $crate::StackTop = {
            static mut STACK: $crate::Stack<STACK_SIZE> = $crate::Stack::new();
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
            "#
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
            "#
        }
    } else {
        compile_error!("unsupported architecture");
    }
}

// // //

const STACK_SIZE: usize =
    sel4_env_literal_helper::env_literal!("SEL4_RUNTIME_STACK_SIZE").unwrap_or(4096 * 4);

declare_stack!(STACK_SIZE);
