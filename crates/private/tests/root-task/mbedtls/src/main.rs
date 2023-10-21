//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use core::ffi::{c_char, c_int, CStr};

use sel4_logging::{LevelFilter, Logger, LoggerBuilder};
use sel4_newlib as _;
use sel4_root_task::{debug_print, debug_println, root_task};

const LOG_LEVEL: LevelFilter = LevelFilter::Debug;

static LOGGER: Logger = LoggerBuilder::const_default()
    .level_filter(LOG_LEVEL)
    .write(|s| debug_println!("{}", s))
    .build();

const HEAP_SIZE: usize = 1024 * 1024;

#[root_task(heap_size = HEAP_SIZE)]
fn main(_: &sel4::BootInfo) -> ! {
    LOGGER.set().unwrap();
    run_tests();
    debug_println!("TEST_PASS");
    sel4::BootInfo::init_thread_tcb().tcb_suspend().unwrap();
    unreachable!()
}

fn run_tests() {
    {
        use sel4_newlib::*;

        set_static_heap_for_sbrk({
            static HEAP: StaticHeap<{ HEAP_SIZE }> = StaticHeap::new();
            &HEAP
        });

        let mut impls = Implementations::default();
        impls._sbrk = Some(sbrk_with_static_heap);
        impls._write = Some(write_with_debug_put_char);
        set_implementations(impls)
    }

    unsafe {
        mbedtls::self_test::enable(rand, Some(log));
    }

    for (name, test) in TESTS {
        if unsafe { (test)(1) } != 0 {
            panic!("{} failed", name)
        }
    }
}

unsafe fn log(msg: *const c_char) {
    debug_print!("{}", CStr::from_ptr(msg).to_string_lossy());
}

fn rand() -> c_int {
    unimplemented!()
}

type Test = unsafe extern "C" fn(c_int) -> c_int;

macro_rules! tests {
    {
        $(
            $(#[$m:meta])*
            $i:ident,
        )*
    } => {
        &[
            $(
                $(#[$m])*
                (stringify!($i), mbedtls::self_test::$i),
            )*
        ]
    };
}

const TESTS: &[(&str, Test)] = tests! {
    aes,
    aria,
    base64,
    camellia,
    ccm,
    chacha20,
    chachapoly,
    ctr_drbg,
    des,
    dhm,
    gcm,
    hmac_drbg,
    md5,
    mpi,
    pkcs5,
    poly1305,
    ripemd160,
    rsa,
    sha1,
    sha224,
    sha256,
    sha384,
    sha512,
    nist_kw,
    cmac,
    ecp,
    ecjpake,
};
