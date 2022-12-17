use core::arch::global_asm;
use core::panic::PanicInfo;
use core::ptr::NonNull;
use core::sync::Exclusive;

use crate::main;

const STACK_SIZE: usize = 4096 * 4;

#[repr(C, align(16))]
struct Stack([u8; STACK_SIZE]);

static mut STACK: Stack = Stack([0; STACK_SIZE]);

#[no_mangle]
static __stack_top: Exclusive<*const u8> = Exclusive::new(unsafe { STACK.0.as_ptr_range().end });

#[no_mangle]
unsafe extern "C" fn __rust_entry(bootinfo: *const sel4::sys::seL4_BootInfo) -> ! {
    let bootinfo = sel4::BootInfo::from_ptr(bootinfo);
    sel4::set_ipc_buffer_ptr(NonNull::new(bootinfo.ipc_buffer()).unwrap());
    main(&bootinfo)
}

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

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    sel4::debug_println!("{}", info);
    let _ = sel4::BootInfo::init_thread_tcb().suspend();
    core::intrinsics::abort()
}
