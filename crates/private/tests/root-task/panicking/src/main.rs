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

#[root_task(stack_size = 4096 * 64, heap_size = 4096 * 16)] // TODO decrease stack size
fn main(_: &sel4::BootInfo) -> ! {
    let _ = panicking::catch_unwind(|| {
        f1();
    });
    assert!(F1_DROPPED.load(Ordering::SeqCst));
    whether_alloc();
    debug_println!("TEST_PASS");

    sel4::BootInfo::init_thread_tcb().tcb_suspend().unwrap();
    unreachable!()
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

cfg_if::cfg_if! {
    if #[cfg(feature = "alloc")] {
        extern crate alloc;

        use alloc::borrow::ToOwned;
        use alloc::string::String;

        fn whether_alloc() {
            let r = panicking::catch_unwind(|| {
                panicking::panic_any::<String>("foo".to_owned());
            });
            assert_eq!(r.err().unwrap().inner().downcast_ref::<String>().unwrap().as_str(), "foo");
        }
    } else {
        use panicking::SmallPayload;

        fn whether_alloc() {
            let r = panicking::catch_unwind(|| {
                panicking::panic_any(Foo(1337));
            });
            assert!(matches!(r.err().unwrap().downcast::<Foo>().ok().unwrap(), Foo(1337)));
        }

        #[derive(Copy, Clone)]
        struct Foo(usize);

        impl SmallPayload for Foo {}
    }
}
