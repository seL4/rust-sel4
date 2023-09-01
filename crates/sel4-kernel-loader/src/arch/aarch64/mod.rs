use core::arch::asm;

pub(crate) mod drivers;
pub(crate) mod exception_handler;
pub(crate) mod init_platform_state;

pub(crate) fn idle() -> ! {
    loop {
        unsafe {
            asm!("wfe");
        }
    }
}
