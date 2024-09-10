//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use core::ptr;

use sel4::CapTypeForObjectOfFixedSize;
use sel4_root_task::{root_task, Never};

mod device;

use device::{Device, RegisterBlock};

const SERIAL_DEVICE_IRQ: usize = 33;

const SERIAL_DEVICE_BASE_ADDR: usize = 0x0900_0000;

#[root_task]
fn main(bootinfo: &sel4::BootInfoPtr) -> sel4::Result<Never> {
    let mut empty_slots = bootinfo
        .empty()
        .range()
        .map(sel4::init_thread::Slot::from_index);

    let kernel_ut = find_largest_kernel_untyped(bootinfo);

    let irq_handler_cap = empty_slots
        .next()
        .unwrap()
        .downcast::<sel4::cap_type::IrqHandler>()
        .cap();

    sel4::init_thread::slot::IRQ_CONTROL
        .cap()
        .irq_control_get(
            SERIAL_DEVICE_IRQ.try_into().unwrap(),
            &sel4::init_thread::slot::CNODE
                .cap()
                .relative(irq_handler_cap),
        )
        .unwrap();

    let irq_notification_slot = empty_slots
        .next()
        .unwrap()
        .downcast::<sel4::cap_type::Notification>();

    kernel_ut
        .untyped_retype(
            &sel4::ObjectBlueprint::Notification,
            &sel4::init_thread::slot::CNODE.cap().relative_self(),
            irq_notification_slot.index(),
            1,
        )
        .unwrap();

    let irq_notification_cap = irq_notification_slot.cap();

    irq_handler_cap
        .irq_handler_set_notification(irq_notification_cap)
        .unwrap();

    let (device_ut_ix, device_ut_desc) = bootinfo
        .untyped_list()
        .iter()
        .enumerate()
        .find(|(_i, desc)| {
            (desc.paddr()..(desc.paddr() + (1 << desc.size_bits())))
                .contains(&SERIAL_DEVICE_BASE_ADDR)
        })
        .unwrap();

    assert!(device_ut_desc.is_device());

    let device_ut_cap = bootinfo.untyped().index(device_ut_ix).cap();

    trim_untyped(
        device_ut_cap,
        device_ut_desc.paddr(),
        SERIAL_DEVICE_BASE_ADDR,
        empty_slots.next().unwrap(),
        empty_slots.next().unwrap(),
    );

    let serial_device_frame_slot = empty_slots
        .next()
        .unwrap()
        .downcast::<sel4::cap_type::Granule>();

    device_ut_cap
        .untyped_retype(
            &sel4::cap_type::Granule::object_blueprint(),
            &sel4::init_thread::slot::CNODE.cap().relative_self(),
            serial_device_frame_slot.index(),
            1,
        )
        .unwrap();

    let serial_device_frame_cap = serial_device_frame_slot.cap();

    let serial_device_frame_addr = init_free_page_addr(bootinfo);

    serial_device_frame_cap
        .frame_map(
            sel4::init_thread::slot::VSPACE.cap(),
            serial_device_frame_addr,
            sel4::CapRights::read_write(),
            sel4::VmAttributes::default(),
        )
        .unwrap();

    let serial_device = unsafe { Device::new(serial_device_frame_addr as *mut RegisterBlock) };

    serial_device.init();

    for c in b"echo> ".iter() {
        serial_device.put_char(*c);
    }

    loop {
        serial_device.clear_all_interrupts();
        irq_handler_cap.irq_handler_ack().unwrap();

        irq_notification_cap.wait();

        while let Some(c) = serial_device.get_char() {
            serial_device.put_char(b'[');
            serial_device.put_char(c);
            serial_device.put_char(b']');
        }
    }
}

// // //

fn find_largest_kernel_untyped(bootinfo: &sel4::BootInfo) -> sel4::cap::Untyped {
    let (ut_ix, _desc) = bootinfo
        .untyped_list()
        .iter()
        .enumerate()
        .filter(|(_i, desc)| !desc.is_device())
        .max_by_key(|(_i, desc)| desc.size_bits())
        .unwrap();

    bootinfo.untyped().index(ut_ix).cap()
}

// // //

fn trim_untyped(
    ut: sel4::cap::Untyped,
    ut_paddr: usize,
    target_paddr: usize,
    free_slot_a: sel4::init_thread::Slot,
    free_slot_b: sel4::init_thread::Slot,
) {
    let rel_a = sel4::init_thread::slot::CNODE
        .cap()
        .relative(free_slot_a.cptr());
    let rel_b = sel4::init_thread::slot::CNODE
        .cap()
        .relative(free_slot_b.cptr());
    let mut cur_paddr = ut_paddr;
    while cur_paddr != target_paddr {
        let size_bits = (target_paddr - cur_paddr).ilog2().try_into().unwrap();
        ut.untyped_retype(
            &sel4::ObjectBlueprint::Untyped { size_bits },
            &sel4::init_thread::slot::CNODE.cap().relative_self(),
            free_slot_b.index(),
            1,
        )
        .unwrap();
        rel_a.delete().unwrap();
        rel_a.move_(&rel_b).unwrap();
        cur_paddr += 1 << size_bits;
    }
}

// // //

#[repr(C, align(4096))]
struct FreePagePlaceHolder(#[allow(dead_code)] [u8; GRANULE_SIZE]);

static mut FREE_PAGE_PLACEHOLDER: FreePagePlaceHolder = FreePagePlaceHolder([0; GRANULE_SIZE]);

fn init_free_page_addr(bootinfo: &sel4::BootInfo) -> usize {
    let addr = ptr::addr_of!(FREE_PAGE_PLACEHOLDER) as usize;
    get_user_image_frame_slot(bootinfo, addr)
        .cap()
        .frame_unmap()
        .unwrap();
    addr
}

fn get_user_image_frame_slot(
    bootinfo: &sel4::BootInfo,
    addr: usize,
) -> sel4::init_thread::Slot<sel4::cap_type::Granule> {
    extern "C" {
        static __executable_start: usize;
    }
    let user_image_addr = ptr::addr_of!(__executable_start) as usize;
    bootinfo
        .user_image_frames()
        .index(addr / GRANULE_SIZE - user_image_addr / GRANULE_SIZE)
}

const GRANULE_SIZE: usize = sel4::FrameObjectType::GRANULE.bytes();
