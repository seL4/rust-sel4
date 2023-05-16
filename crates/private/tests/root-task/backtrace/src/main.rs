#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;

use sel4_backtrace::collect;
use sel4_backtrace_embedded_debug_info::get_context;
use sel4_backtrace_simple::SimpleBacktracing;
use sel4_root_task_runtime::{debug_println, main, panicking};

// TODO
// Why are such a large stack and heap required? The unwinding part seems to consume the stack, and
// addr2line the heap.
#[main(stack_size = 4096 * 64, heap_size = 16 << 20)]
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

    let mut s = String::new();
    collect(())
        .symbolize(&get_context().unwrap(), &mut s)
        .unwrap();
    debug_println!("{}", s);
}
