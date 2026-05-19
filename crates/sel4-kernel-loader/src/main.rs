//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]
#![allow(dead_code)]

use core::hint;
use core::sync::atomic::{AtomicBool, Ordering};

use sel4_config::sel4_cfg_usize;
use sel4_immediate_sync_once_cell::ImmediateSyncOnceCell;
use sel4_kernel_loader_payload_types::ArchivedPayloadInfo;

use sel4_no_allocator as _;

mod arch;
mod barrier;
mod enter_kernel;
mod fmt;
mod init_memory;
mod logging;
mod plat;
mod rt;
mod this_image;

use crate::{
    arch::{Arch, ArchImpl},
    barrier::Barrier,
    enter_kernel::mk_enter_kernel,
    fmt::debug_println,
    plat::{Plat, PlatImpl},
};

const MAX_NUM_NODES: usize = sel4_cfg_usize!(MAX_NUM_NODES);

static PAYLOAD_INFO: ImmediateSyncOnceCell<ArchivedPayloadInfo> = ImmediateSyncOnceCell::new();

static SECONDARY_READY: AtomicBool = AtomicBool::new(false);

#[allow(clippy::reversed_empty_ranges)]
fn main(physical_core_id: usize) -> ! {
    PlatImpl::init();

    logging::set_logger();

    log::info!("Starting loader on physical core {physical_core_id}");

    PAYLOAD_INFO.set(init_memory::init()).unwrap();

    for other_core_id in 0..MAX_NUM_NODES {
        let other_physical_core_id = ArchImpl::logical_to_physical_core_id(other_core_id);
        if other_physical_core_id != physical_core_id {
            log::debug!("Starting core {other_physical_core_id}");
            SECONDARY_READY.store(false, Ordering::SeqCst);
            let sp = this_image::stacks::get_secondary_stack_bottom(other_core_id)
                .ptr()
                .addr();
            PlatImpl::start_core(other_physical_core_id, sp);
            while !SECONDARY_READY.load(Ordering::SeqCst) {
                hint::spin_loop();
            }
            log::debug!("Core {other_physical_core_id} up");
        }
    }

    if let Some(core_id) = ArchImpl::physical_to_logical_core_id(physical_core_id) {
        common_epilogue(core_id)
    } else {
        PlatImpl::stop_core()
    }
}

fn secondary_main(physical_core_id: usize) -> ! {
    let core_id = ArchImpl::physical_to_logical_core_id(physical_core_id).unwrap();
    log::debug!("Core {core_id}: up");
    SECONDARY_READY.store(true, Ordering::SeqCst);
    common_epilogue(core_id)
}

static KERNEL_ENTRY_BARRIER: Barrier = Barrier::new(MAX_NUM_NODES);

fn common_epilogue(core_id: usize) -> ! {
    PlatImpl::init_per_core();
    let payload_info = PAYLOAD_INFO.get().unwrap();
    let enter_kernel = mk_enter_kernel(payload_info, core_id);
    if core_id == 0 {
        log::info!("Entering kernel");
    }
    KERNEL_ENTRY_BARRIER.wait();
    ArchImpl::prepare_to_enter_kernel(core_id);
    enter_kernel();
    debug_println!("Core {}: failed to enter kernel", core_id);
    ArchImpl::idle()
}
