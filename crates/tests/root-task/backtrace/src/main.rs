#![no_std]
#![no_main]

use sel4_backtrace_simple::SimpleBacktracing;
use sel4_root_task_runtime::{debug_println, main, panicking};

#[main(stack_size = 4096 * 64, heap_size = 4096 * 16)] // TODO decrease stack size
fn main(_: &sel4::BootInfo) -> ! {
    let _ = panicking::catch_unwind(|| {
        f();
    });
    debug_println!("TEST_PASS");

    sel4::BootInfo::init_thread_tcb().tcb_suspend().unwrap();
    unreachable!()
}

pub fn f() {
    [()].iter().for_each(g);
}

fn g(_: &()) -> () {
    let simple = SimpleBacktracing::new(None);
    let bt = simple.collect();
    simple.send(&bt);
    assert!(bt.postamble.error.is_none());
    assert_eq!(bt.entries.len(), 26);
}
