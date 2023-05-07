use core::ops::Range;

use loader_types::{Payload, PayloadInfo, RegionContent};
use sel4_platform_info::PLATFORM_INFO;

use crate::{
    barrier::Barrier, copy_payload_data, debug, enter_kernel, fmt, idle, init_platform_state,
    logging, sanity_check, smp, MAX_NUM_NODES,
};

static KERNEL_ENTRY_BARRIER: Barrier = Barrier::new(MAX_NUM_NODES);

pub(crate) fn run<T: RegionContent, const N: usize>(
    get_payload: impl FnOnce() -> (Payload<T, N>, &'static T::Source),
    own_footprint: &Range<usize>,
) -> !
where
    <T as RegionContent>::Source: 'static,
{
    debug::init();

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

    {
        let own_footprint =
            own_footprint.start.try_into().unwrap()..own_footprint.end.try_into().unwrap();
        sanity_check::sanity_check(&own_footprint, &payload.data);
    }

    log::debug!("Copying payload data");
    copy_payload_data::copy_payload_data(&payload.data, region_content_source);

    smp::start_secondary_cores(&payload.info);

    common_epilogue(0, &payload.info)
}

pub(crate) fn secondary_main(core_id: usize, payload_info: &PayloadInfo) -> ! {
    common_epilogue(core_id, payload_info)
}

pub(crate) fn common_epilogue(core_id: usize, payload_info: &PayloadInfo) -> ! {
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
