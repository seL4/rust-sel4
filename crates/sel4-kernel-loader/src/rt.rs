use core::panic::PanicInfo;

use crate::arch::idle;

#[panic_handler]
extern "C" fn panic_handler(info: &PanicInfo) -> ! {
    log::error!("{}", info);
    idle()
}
