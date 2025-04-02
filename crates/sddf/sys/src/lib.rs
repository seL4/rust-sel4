//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![allow(non_camel_case_types)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// TODO
pub const SDDF_SERIAL_MAGIC: [core::ffi::c_char; SDDF_SERIAL_MAGIC_LEN as usize] =
    [b's', b'D', b'D', b'F', 0x3];

// TODO
pub const SDDF_TIMER_MAGIC: [core::ffi::c_char; SDDF_TIMER_MAGIC_LEN as usize] =
    [b's', b'D', b'D', b'F', 0x6];

const _: () = {
    use ptr_meta::Pointee;

    macro_rules! impls {
        ($($t:ident,)*) => {
            $(
                unsafe impl<T> Pointee for $t<[T]> {
                    type Metadata = <[T] as Pointee>::Metadata;
                }
            )*
        }
    }

    impls![
        blk_req_queue,
        blk_resp_queue,
        gpu_req_queue,
        gpu_resp_queue,
        net_queue,
    ];
};
