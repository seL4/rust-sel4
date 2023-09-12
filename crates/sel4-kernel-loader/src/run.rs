use core::ops::Range;

use sel4_kernel_loader_payload_types::{Payload, PayloadInfo, RegionContent};
use sel4_platform_info::PLATFORM_INFO;

use crate::{
    arch::{idle, init_platform_state},
    barrier::Barrier,
    enter_kernel, fmt, logging, plat, smp, MAX_NUM_NODES,
};

static KERNEL_ENTRY_BARRIER: Barrier = Barrier::new(MAX_NUM_NODES);

pub(crate) fn run<U: RegionContent<Source: 'static>, const N: usize>(
    get_payload: impl FnOnce() -> (Payload<usize, U, N>, &'static U::Source),
    own_footprint: &Range<usize>,
) -> ! {
    plat::debug::init();

    logging::set_logger();

    log::info!("Starting loader");

    let (payload, region_content_source) = get_payload();

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

    smp::start_secondary_cores(&payload.info);

    common_epilogue(0, &payload.info)
}

pub(crate) fn secondary_main(core_id: usize, payload_info: &PayloadInfo<usize>) -> ! {
    common_epilogue(core_id, payload_info)
}

pub(crate) fn common_epilogue(core_id: usize, payload_info: &PayloadInfo<usize>) -> ! {
    if core_id == 0 {
        log::info!("Entering kernel");
    }
    KERNEL_ENTRY_BARRIER.wait();
    init_platform_state::init_platform_state_per_core(core_id);
    init_platform_state::init_platform_state_per_core_after_which_no_syncronization(core_id);
    enter_kernel::enter_kernel(payload_info);
    fmt::debug_println_without_synchronization!("Core {}: failed to enter kernel", core_id);
    idle()
}
