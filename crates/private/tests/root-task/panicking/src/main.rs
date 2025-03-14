//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, Ordering};

use sel4_root_task::{debug_println, panicking, root_task};

static F1_DROPPED: AtomicBool = AtomicBool::new(false);

#[root_task(heap_size = 4096 * 16)]
fn main(_: &sel4::BootInfoPtr) -> ! {
    let _ = panicking::catch_unwind(|| {
        f1();
    });
    assert!(F1_DROPPED.load(Ordering::SeqCst));
    debug_println!("TEST_PASS");

    sel4::init_thread::suspend_self()
}

fn f1() {
    [()].iter().for_each(f1_helper);
}

fn f1_helper(_: &()) {
    let _ = F1Drop;
    panic!("test");
}

struct F1Drop;

impl Drop for F1Drop {
    fn drop(&mut self) {
        debug_println!("F1Drop::drop()");
        F1_DROPPED.store(true, Ordering::SeqCst);
    }
}
