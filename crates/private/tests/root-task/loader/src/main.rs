#![no_std]
#![no_main]
#![feature(is_sorted)]
#![feature(iter_intersperse)]
#![feature(thread_local)]
#![allow(clippy::single_match)]

use sel4_platform_info::PLATFORM_INFO;
use sel4_root_task::{debug_print, debug_println, root_task};

#[repr(C, align(8192))]
struct Y(i32);

#[no_mangle]
#[thread_local]
static X: Y = Y(1337);

#[root_task]
fn main(bootinfo: &sel4::BootInfo) -> ! {
    debug_println!("{}", X.0);
    assert_eq!(X.0, 1337);

    debug_println!("Gaps in device untypeds:");
    let mut last_end = 0;
    for ut in bootinfo.device_untyped_list() {
        if ut.paddr() > last_end {
            debug_println!("{:x?}", last_end..ut.paddr());
        }
        last_end = ut.paddr() + (1 << ut.size_bits());
    }

    debug_println!("Gaps in kernel untypeds:");
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
