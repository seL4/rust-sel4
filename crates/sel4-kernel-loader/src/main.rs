#![no_std]
#![no_main]
#![feature(associated_type_bounds)]
#![feature(atomic_from_mut)]
#![feature(const_pointer_byte_offsets)]
#![feature(const_trait_impl)]
#![feature(exclusive_wrapper)]
#![feature(pointer_byte_offsets)]
#![feature(proc_macro_hygiene)]
#![feature(strict_provenance)]
#![feature(stdsimd)]
#![allow(dead_code)]
#![allow(unreachable_code)]

use core::mem;

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

use crate::barrier::Barrier;

const MAX_NUM_NODES: usize = sel4_config::sel4_cfg_usize!(MAX_NUM_NODES);

const NUM_SECONDARY_CORES: usize = MAX_NUM_NODES - 1;

#[no_mangle]
extern "C" fn main() -> ! {
    plat::debug::init();

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

    start_secondary_cores(&payload.info);

    common_epilogue(0, &payload.info)
}

fn secondary_main(core_id: usize, payload_info: &PayloadInfo<usize>) -> ! {
    common_epilogue(core_id, payload_info)
}

static KERNEL_ENTRY_BARRIER: Barrier = Barrier::new(MAX_NUM_NODES);

fn common_epilogue(core_id: usize, payload_info: &PayloadInfo<usize>) -> ! {
    if core_id == 0 {
        log::info!("Entering kernel");
    }
    KERNEL_ENTRY_BARRIER.wait();
    arch::init_platform_state_per_core(core_id);
    arch::init_platform_state_per_core_after_which_no_syncronization(core_id);
    enter_kernel(payload_info);
    fmt::debug_println_without_synchronization!("Core {}: failed to enter kernel", core_id);
    arch::idle()
}

type KernelEntry = extern "C" fn(
    ui_p_reg_start: usize,
    ui_p_reg_end: usize,
    pv_offset: isize,
    v_entry: usize,
    dtb_addr_p: usize,
    dtb_size: usize,
) -> !;

fn enter_kernel(payload_info: &PayloadInfo<usize>) -> ! {
    let kernel_entry =
        unsafe { mem::transmute::<usize, KernelEntry>(payload_info.kernel_image.virt_entry) };

    let (dtb_addr_p, dtb_size) = match &payload_info.fdt_phys_addr_range {
        Some(region) => (region.start, region.len()),
        None => (0, 0),
    };

    (kernel_entry)(
        payload_info.user_image.phys_addr_range.start,
        payload_info.user_image.phys_addr_range.end,
        0_usize.wrapping_sub(payload_info.user_image.phys_to_virt_offset) as isize,
        payload_info.user_image.virt_entry,
        dtb_addr_p,
        dtb_size,
    )
}

static SECONDARY_CORE_INIT_INFO: RwLock<Option<SecondaryCoreInitInfo>> = RwLock::new(None);

struct SecondaryCoreInitInfo {
    core_id: usize,
    payload_info: PayloadInfo<usize>,
    barrier: Barrier,
}

fn start_secondary_cores(payload_info: &PayloadInfo<usize>) {
    for i in 0..NUM_SECONDARY_CORES {
        let core_id = i + 1;
        let sp = this_image::stacks::get_secondary_stack_bottom(i);
        {
            let mut init_info = SECONDARY_CORE_INIT_INFO.write();
            *init_info = Some(SecondaryCoreInitInfo {
                core_id,
                payload_info: payload_info.clone(),
                barrier: Barrier::new(2),
            });
        }
        log::debug!("Primary core: starting core {}", core_id);
        plat::start_secondary_core(core_id, sp);
        {
            let init_info = SECONDARY_CORE_INIT_INFO.read();
            let init_info = init_info.as_ref().unwrap();
            init_info.barrier.wait();
        }
        log::debug!("Primary core: core {} up", core_id);
    }
}

#[no_mangle]
extern "C" fn secondary_entry_rust() -> ! {
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
    secondary_main(core_id, &payload_info)
}
