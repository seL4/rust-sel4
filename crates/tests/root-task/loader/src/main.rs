#![no_std]
#![no_main]
#![feature(is_sorted)]
#![feature(iter_intersperse)]
#![feature(thread_local)]

use sel4_platform_info::PLATFORM_INFO;
use sel4_root_task_runtime::{debug_print, debug_println, main};

#[repr(C, align(8192))]
struct Y(i32);

#[no_mangle]
#[thread_local]
static X: Y = Y(1337);

#[main]
fn main(bootinfo: &sel4::BootInfo) -> ! {
    debug_println!("{}", X.0);
    assert_eq!(X.0, 1337);

    assert!(bootinfo
        .device_untyped_list()
        .is_sorted_by_key(|ut| ut.paddr()));
    assert!(bootinfo
        .kernel_untyped_list()
        .is_sorted_by_key(|ut| ut.paddr()));

    let mut last_end = PLATFORM_INFO.memory[0].start.try_into().unwrap();
    for ut in bootinfo.kernel_untyped_list() {
        if ut.paddr() > last_end {
            debug_println!("{:x?}", last_end..ut.paddr());
        }
        last_end = ut.paddr() + (1 << ut.size_bits());
    }

    for extra in bootinfo.extra() {
        match extra.id {
            sel4::BootInfoExtraId::Fdt => {
                let dt = fdt::Fdt::new(extra.content()).unwrap();
                for s in dt.strings().intersperse(" ") {
                    debug_print!("{}", s);
                }
                debug_println!("");
            }
            _ => {}
        }
    }

    // for ut in bootinfo.kernel_untyped_list() {
    //     debug_println!("k {:x?} {}", ut.paddr, ut.isDevice);
    // }

    // for ut in bootinfo.device_untyped_list() {
    //     debug_println!("d {:x?} {}", ut.paddr, ut.isDevice);
    // }

    debug_println!("TEST_PASS");

    sel4::BootInfo::init_thread_tcb().tcb_suspend().unwrap();
    unreachable!()
}
