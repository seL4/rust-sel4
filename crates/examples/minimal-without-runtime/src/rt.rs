use core::arch::{asm, global_asm};
use core::sync::Exclusive;
use core::panic::PanicInfo;
use core::ptr::NonNull;

use crate::main;

const STACK_SIZE: usize = 4096 * 4;

#[repr(C, align(16))]
struct Stack([u8; STACK_SIZE]);

static mut STACK: Stack = Stack([0; STACK_SIZE]);

#[no_mangle]
static __stack_top: Exclusive<*const u8> = Exclusive::new(unsafe { STACK.0.as_ptr_range().end });

const TLS_REGION_SIZE: usize = 4096;

// [HACK] Assume max alignment for now
// [HACK] Assume TLS filesz == 0 for now
#[repr(C, align(4096))]
struct TLSRegion([u8; TLS_REGION_SIZE]);

static mut TLS_REGION: TLSRegion = TLSRegion([0; TLS_REGION_SIZE]);

#[no_mangle]
unsafe extern "C" fn __rust_entry(bootinfo: *const sel4::sys::seL4_BootInfo) -> ! {
    set_tls_base();
    let ipc_buffer = (*bootinfo).ipcBuffer;
    sel4::set_ipc_buffer_ptr(NonNull::new(ipc_buffer).unwrap());
    let bootinfo = sel4::BootInfo::from_ptr(bootinfo);
    main(&bootinfo)
}

#[inline(never)] 
unsafe fn set_tls_base() {
    let tls_base = TLS_REGION.0.as_mut_ptr();
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "aarch64")] {
            asm!("msr tpidr_el0, {tpidr}", tpidr = in(reg) tls_base);
        } else if #[cfg(target_arch = "x86_64")] {
            sel4::sys::seL4_SetTLSBase(tls_base.to_bits().try_into().unwrap());
        } else {
            compile_error!("unsupported architecture");
        }
    }
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
    core::intrinsics::abort()
}
