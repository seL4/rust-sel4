use core::arch::asm;

pub(crate) mod init_platform_state;

pub(crate) fn idle() -> ! {
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}

#[no_mangle]
static mut hsm_exists: i32 = 0;
