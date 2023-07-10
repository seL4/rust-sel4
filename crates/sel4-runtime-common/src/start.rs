// TODO
// - guard pages for stack?

use core::arch::global_asm;
use core::cell::UnsafeCell;
use core::sync::Exclusive;

// TODO alignment should depend on configuration
#[repr(C, align(16))]
pub struct Stack<const N: usize>(UnsafeCell<[u8; N]>);

impl<const N: usize> Stack<N> {
    pub const fn new() -> Self {
        Self(UnsafeCell::new([0; N]))
    }

    pub const fn top(&self) -> StackTop {
        StackTop(Exclusive::new(self.0.get().cast::<u8>().wrapping_add(N)))
    }
}

#[repr(transparent)]
pub struct StackTop(Exclusive<*mut u8>);

#[macro_export]
macro_rules! declare_stack {
    ($size:expr) => {
        #[no_mangle]
        static __sel4_runtime_stack_top: $crate::_private::start::StackTop = {
            static mut STACK: $crate::_private::start::Stack<{ $size }> =
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

pub mod _private {
    pub use super::{Stack, StackTop};
}
