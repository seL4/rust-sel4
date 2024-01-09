//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]
#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(clippy::reversed_empty_ranges)]

use spin::RwLock;

use sel4_kernel_loader_payload_types::PayloadInfo;
use sel4_platform_info::PLATFORM_INFO;

mod arch;
mod barrier;
mod drivers;
mod fmt;
mod logging;
mod plat;
mod rt;
mod this_image;

use crate::{
    arch::{Arch, ArchImpl},
    barrier::Barrier,
    plat::{Plat, PlatImpl},
};

const MAX_NUM_NODES: usize = sel4_config::sel4_cfg_usize!(MAX_NUM_NODES);

static SECONDARY_CORE_INIT_INFO: RwLock<Option<SecondaryCoreInitInfo>> = RwLock::new(None);

struct SecondaryCoreInitInfo {
    core_id: usize,
    payload_info: PayloadInfo<usize>,
    barrier: Barrier,
}

fn main(per_core: <ArchImpl as Arch>::PerCore) -> ! {
    ArchImpl::init();
    PlatImpl::init();

    logging::set_logger();

    log::info!("Starting loader");

    let (payload, region_content_source) = this_image::get_payload();

    let own_footprint = this_image::get_user_image_bounds();

    log::debug!("Platform info: {:#x?}", PLATFORM_INFO);
    log::debug!("Loader footprint: {:#x?}", own_footprint);
    log::debug!("Payload info: {:#x?}", payload.info);
    log::debug!("Payload regions:");
    for region in payload.data.iter() {
        log::debug!(
            "    0x{:x?} {:?}",
            region.phys_addr_range,
            region.content.is_some()
        );
    }

    payload.sanity_check(&PLATFORM_INFO, own_footprint.clone());

    log::debug!("Copying payload data");
    unsafe {
        payload.copy_data_out(region_content_source);
    }

    for core_id in 1..MAX_NUM_NODES {
        let sp = this_image::stacks::get_secondary_stack_bottom(core_id);
        {
            let mut init_info = SECONDARY_CORE_INIT_INFO.write();
            *init_info = Some(SecondaryCoreInitInfo {
                core_id,
                payload_info: payload.info.clone(),
                barrier: Barrier::new(2),
            });
        }
        log::debug!("Primary core: starting core {}", core_id);
        PlatImpl::start_secondary_core(core_id, sp);
        {
            let init_info = SECONDARY_CORE_INIT_INFO.read();
            let init_info = init_info.as_ref().unwrap();
            init_info.barrier.wait();
        }
        log::debug!("Primary core: core {} up", core_id);
    }

    common_epilogue(0, &payload.info, per_core)
}

fn secondary_main(per_core: <ArchImpl as Arch>::PerCore) -> ! {
    let core_id;
    let payload_info;
    {
        let init_info = SECONDARY_CORE_INIT_INFO.read();
        let init_info = init_info.as_ref().unwrap();
        init_info.barrier.wait();
        core_id = init_info.core_id;
        payload_info = init_info.payload_info.clone();
    }
    log::debug!("Core {}: up", core_id);
    common_epilogue(core_id, &payload_info, per_core)
}

static KERNEL_ENTRY_BARRIER: Barrier = Barrier::new(MAX_NUM_NODES);

fn common_epilogue(
    core_id: usize,
    payload_info: &PayloadInfo<usize>,
    per_core: <ArchImpl as Arch>::PerCore,
) -> ! {
    PlatImpl::init_per_core();
    if core_id == 0 {
        log::info!("Entering kernel");
    }
    KERNEL_ENTRY_BARRIER.wait();
    ArchImpl::enter_kernel(core_id, payload_info, per_core);
    fmt::debug_println_without_synchronization!("Core {}: failed to enter kernel", core_id);
    ArchImpl::idle()
}
