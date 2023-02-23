#![no_std]
#![no_main]
#![feature(const_pointer_byte_offsets)]
#![feature(pointer_byte_offsets)]

use core::ops::Range;

use heapless::Vec;

use loader_payload_types::*;

#[no_mangle]
extern "C" fn main() -> ! {
    let payload = get_payload();
    let payload = Payload {
        info: payload.info,
        data: payload.data.as_slice(),
    };
    loader_core::main(&payload, &get_own_footprint())
}

fn get_own_footprint() -> Range<usize> {
    unsafe { (&__executable_start as *const i32 as usize)..(&_end as *const i32 as usize) }
}

extern "C" {
    static __executable_start: i32;
    static _end: i32;
}

mod translation_tables {
    include!(concat!(env!("OUT_DIR"), "/translation_tables.rs"));
}

const MAX_NUM_REGIONS: usize = 16;

fn get_payload() -> Payload<Vec<Region<&'static [u8]>, MAX_NUM_REGIONS>> {
    todo!()
}
