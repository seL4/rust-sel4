//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, Ordering};

use sel4_microkit::{NullHandler, debug_println, panicking, protection_domain};

sel4_test_microkit::embed_sdf_xml!("system.xml");

static F1_DROPPED: AtomicBool = AtomicBool::new(false);

#[protection_domain]
fn init() -> NullHandler {
    let _ = panicking::catch_unwind(|| {
        f1();
    });
    assert!(F1_DROPPED.load(Ordering::SeqCst));
    sel4_test_microkit::indicate_success()
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
