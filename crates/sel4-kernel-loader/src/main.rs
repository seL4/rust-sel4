//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]
#![allow(dead_code)]

use core::hint;
use core::sync::atomic::{AtomicUsize, Ordering};

use sel4_immediate_sync_once_cell::ImmediateSyncOnceCell;
use sel4_kernel_loader_payload_types::ArchivedPayloadInfo;
use sel4_platform_info::PLATFORM_INFO;

use sel4_no_allocator as _;

mod arch;
mod barrier;
mod enter_kernel;
mod fmt;
mod logging;
mod plat;
mod rt;
mod this_image;

use crate::{
    arch::{Arch, ArchImpl},
    barrier::Barrier,
    enter_kernel::{KernelEntryExtraArgs, mk_enter_kernel},
    plat::{Plat, PlatImpl},
};

const MAX_NUM_NODES: usize = sel4_config::sel4_cfg_usize!(MAX_NUM_NODES);

static PAYLOAD_INFO: ImmediateSyncOnceCell<ArchivedPayloadInfo> = ImmediateSyncOnceCell::new();

static NODES_STARTED: AtomicUsize = AtomicUsize::new(0);
static NODES_READY: AtomicUsize = AtomicUsize::new(0);

#[allow(clippy::reversed_empty_ranges)]
fn main(kernel_entry_extra_args: KernelEntryExtraArgs) -> ! {
    PlatImpl::init();

    logging::set_logger();

    log::info!("Starting loader");

    let payload = this_image::get_payload();

    let own_footprint = this_image::get_user_image_bounds();

    log::debug!("Platform info: {PLATFORM_INFO:#x?}");
    log::debug!("Loader footprint: {own_footprint:#x?}");
    log::debug!("Payload info: {:#x?}", payload.info);
    log::debug!("Payload regions:");
    for region in payload.data.iter() {
        log::debug!(
            "    {:#x?} (filesz = {:#x?}, memsz = {:#x?})",
            region.addr.0,
            region.size.0,
            region.data.len()
        );
    }

    payload.sanity_check(&PLATFORM_INFO, own_footprint.clone());

    log::debug!("Copying payload data");
    unsafe {
        payload.copy_data_out();
    }

    PAYLOAD_INFO.set(payload.info.clone()).unwrap();

    for core_id in 1..MAX_NUM_NODES {
        let sp = this_image::stacks::get_secondary_stack_bottom(core_id).ptr() as usize;
        log::debug!("Primary core: starting core {core_id}");
        NODES_STARTED.store(core_id, Ordering::SeqCst);
        PlatImpl::start_secondary_core(core_id, sp);
        while NODES_READY.load(Ordering::SeqCst) != core_id {
            hint::spin_loop();
        }
        log::debug!("Primary core: core {core_id} up");
    }

    common_epilogue(0, kernel_entry_extra_args)
}

fn secondary_main(kernel_entry_extra_args: KernelEntryExtraArgs) -> ! {
    let core_id = NODES_STARTED.load(Ordering::SeqCst);
    log::debug!("Core {core_id}: up");
    NODES_READY.store(core_id, Ordering::SeqCst);
    common_epilogue(core_id, kernel_entry_extra_args)
}

static KERNEL_ENTRY_BARRIER: Barrier = Barrier::new(MAX_NUM_NODES);

#[allow(unreachable_code)]
fn common_epilogue(core_id: usize, kernel_entry_extra_args: KernelEntryExtraArgs) -> ! {
    PlatImpl::init_per_core();
    let payload_info = PAYLOAD_INFO.get().unwrap();
    let enter_kernel = mk_enter_kernel(payload_info, core_id, kernel_entry_extra_args);
    if core_id == 0 {
        log::info!("Entering kernel");
    }
    KERNEL_ENTRY_BARRIER.wait();
    ArchImpl::prepare_to_enter_kernel(core_id);
    enter_kernel();
    fmt::debug_println_without_synchronization!("Core {}: failed to enter kernel", core_id);
    ArchImpl::idle()
}
