use sel4_runtime_building_blocks_abort::abort;

use crate::Payload;

pub(crate) fn panic_cleanup(_exception: *mut u8) -> Payload {
    unreachable!()
}

pub(crate) fn start_panic(_payload: Payload) -> i32 {
    abort()
}
