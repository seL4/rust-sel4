use sel4_panicking_env::abort;

use crate::Payload;

pub(crate) fn panic_cleanup(_exception: *mut u8) -> Payload {
    unreachable!()
}

pub(crate) fn start_panic(_payload: Payload) -> i32 {
    abort()
}

#[cfg(panic = "unwind")]
#[lang = "eh_personality"]
extern "C" fn personality() -> ! {
    abort!("unexpected call to eh_personality")
}
