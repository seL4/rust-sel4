use core::arch::global_asm;

const STACK_SIZE: usize = 4096 * 4;

#[repr(C, align(16))]
struct Stack([u8; STACK_SIZE]);

#[no_mangle]
static mut __stack: Stack = Stack([0; STACK_SIZE]);

#[no_mangle]
static __stack_size: usize = STACK_SIZE;

cfg_if::cfg_if! {
    if #[cfg(target_arch = "aarch64")] {
        global_asm! {
            concat!(r#"
                .extern main
                .extern __stack
                .extern __stack_size

                .global _start

                .section .text

                _start:
                    ldr x9, =__stack
                    ldr x10, =__stack_size
                    ldr x10, [x10]
                    add sp, x9, x10
                    b main
            "#)
        }
    } else {
        compile_error!("unsupported architecture");
    }
}
