#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, Ordering};

use sel4_full_root_task_runtime::{catch_unwind, debug_println, main};

static F1_DROPPED: AtomicBool = AtomicBool::new(false);

#[main]
fn main(_: &sel4::BootInfo) -> ! {
    let _ = catch_unwind(|| {
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

fn f1_helper(_: &()) -> () {
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
            let r = catch_unwind(|| {
                sel4_panicking::panic_any("foo".to_owned());
            });
            assert_eq!(r.err().unwrap().0.unwrap().downcast_ref::<String>().unwrap().as_str(), "foo");
        }
    } else {
        use sel4_panicking::Region;

        fn whether_alloc() {
            let r = catch_unwind(|| {
                sel4_panicking::panic_val(1337);
            });
            assert!(matches!(r.err().unwrap().0.unwrap(), Region::Val(1337)));
        }
    }
}
