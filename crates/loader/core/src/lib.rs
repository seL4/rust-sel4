#![no_std]
#![no_main]
#![feature(atomic_from_mut)]
#![feature(const_pointer_byte_offsets)]
#![feature(const_trait_impl)]
#![feature(exclusive_wrapper)]
#![feature(pointer_byte_offsets)]
#![feature(strict_provenance)]
#![allow(dead_code)]
#![allow(unreachable_code)]

use core::arch::asm;
use core::ops::Range;
use core::panic::PanicInfo;

use loader_types::{Payload, PayloadInfo, RegionContent};
use sel4_logging::LevelFilter;
use sel4_platform_info::PLATFORM_INFO;

mod barrier;
mod copy_payload_data;
mod debug;
mod drivers;
mod enter_kernel;
mod exception_handler;
mod fmt;
mod init_platform_state;
mod logging;
mod plat;
mod sanity_check;
mod smp;
mod stacks;

use barrier::Barrier;

const LOG_LEVEL: LevelFilter = LevelFilter::Debug;

const MAX_NUM_NODES: usize = sel4_config::sel4_cfg_usize!(MAX_NUM_NODES);
const NUM_SECONDARY_CORES: usize = MAX_NUM_NODES - 1;

static KERNEL_ENTRY_BARRIER: Barrier = Barrier::new(MAX_NUM_NODES);

pub fn main<T: RegionContent, const N: usize>(
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

fn secondary_main(core_id: usize, payload_info: &PayloadInfo) -> ! {
    common_epilogue(core_id, payload_info)
}

fn common_epilogue(core_id: usize, payload_info: &PayloadInfo) -> ! {
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

//

#[panic_handler]
extern "C" fn panic_handler(info: &PanicInfo) -> ! {
    log::error!("{}", info);
    idle()
}

fn idle() -> ! {
    loop {
        unsafe {
            asm!("wfe");
        }
    }
}

//

mod translation_tables {
    include!(concat!(env!("OUT_DIR"), "/translation_tables.rs"));
}
