//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

// TODO address thread safety and reentrancy (init reentrancy structs and figure out what's up with errno)

#![no_std]

#[allow(unused_imports)]
use core::ffi::{c_char, c_int, c_uint};

mod errno;
mod heap;

pub use heap::StaticHeap;

extern "C" {
    #[link_name = "srand"]
    fn newlib_srand(seed: c_uint);
}

pub fn srand(seed: c_uint) {
    unsafe {
        newlib_srand(seed);
    }
}

#[cfg(feature = "_exit")]
mod impl_exit {
    use super::*;

    use sel4_panicking_env::abort;

    #[no_mangle]
    extern "C" fn _exit(rc: c_int) -> ! {
        abort!("_exit({})", rc)
    }
}

#[cfg(feature = "_write")]
mod impl_write {
    use super::*;

    use core::slice;

    use sel4_panicking_env::debug_put_char;

    #[no_mangle]
    extern "C" fn _write(file: c_int, ptr: *const c_char, len: c_int) -> c_int {
        match file {
            1 | 2 => {
                let bytes =
                    unsafe { slice::from_raw_parts(ptr.cast::<u8>(), len.try_into().unwrap()) };
                for &b in bytes {
                    debug_put_char(b);
                }
                len
            }
            _ => {
                #[cfg(feature = "log")]
                {
                    log::warn!("_write({}, {:#x?}, {})", file, ptr, len);
                }
                errno::set_errno(errno::values::ENOENT);
                -1
            }
        }
    }
}
