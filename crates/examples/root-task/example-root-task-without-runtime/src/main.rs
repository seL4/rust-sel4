//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

mod runtime;

enum Never {}

fn main(bootinfo: &sel4::BootInfoPtr) -> sel4::Result<Never> {
    sel4::debug_println!("Hello, World!");

    let mut ipc_buffer = unsafe { bootinfo.ipc_buffer().as_mut().unwrap() };

    let blueprint = sel4::ObjectBlueprint::Notification;

    let chosen_untyped_ix = bootinfo
        .untyped_list()
        .iter()
        .position(|desc| !desc.is_device() && desc.size_bits() >= blueprint.physical_size_bits())
        .unwrap();

    let untyped = bootinfo.untyped().index(chosen_untyped_ix).cap();

    let mut empty_slots = bootinfo
        .empty()
        .range()
        .map(sel4::init_thread::Slot::from_index);
    let unbadged_notification_slot = empty_slots.next().unwrap();
    let badged_notification_slot = empty_slots.next().unwrap();

    let cnode = sel4::init_thread::slot::CNODE.cap();

    untyped.with(&mut ipc_buffer).untyped_retype(
        &blueprint,
        &cnode.absolute_cptr_for_self(),
        unbadged_notification_slot.index(),
        1,
    )?;

    let badge = 0x1337;

    cnode
        .with(&mut ipc_buffer)
        .absolute_cptr(badged_notification_slot.cptr())
        .mint(
            &cnode.absolute_cptr(unbadged_notification_slot.cptr()),
            sel4::CapRights::write_only(),
            badge,
        )?;

    let unbadged_notification = unbadged_notification_slot
        .downcast::<sel4::cap_type::Notification>()
        .cap();
    let badged_notification = badged_notification_slot
        .downcast::<sel4::cap_type::Notification>()
        .cap();

    badged_notification.with(&mut ipc_buffer).signal();

    let (_, observed_badge) = unbadged_notification.with(&mut ipc_buffer).wait();

    sel4::debug_println!("badge = {:#x}", badge);
    assert_eq!(observed_badge, badge);

    sel4::debug_println!("TEST_PASS");

    sel4::init_thread::slot::TCB
        .cap()
        .with(&mut ipc_buffer)
        .tcb_suspend()
        .unwrap();

    unreachable!()
}
