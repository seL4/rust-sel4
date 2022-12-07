#![no_std]
#![no_main]
#![feature(atomic_from_mut)]
#![feature(ptr_to_from_bits)]
#![allow(unreachable_code)]
#![allow(dead_code)]

use core::ops::Range;
use core::panic::PanicInfo;

use aarch64_cpu::asm::wfe;
use log::LevelFilter;
use spin::Barrier;

use loader_payload_types::{Payload, PayloadInfo};
use sel4_platform_info::PLATFORM_INFO;

mod copy_payload_data;
mod debug;
mod drivers;
mod enter_kernel;
mod exception_handler;
mod fmt;
mod head;
mod init_platform_state;
mod init_translation_structures;
mod logging;
mod plat;
mod smp;

use logging::Logger;

const LOG_LEVEL: LevelFilter = LevelFilter::Info;

static LOGGER: Logger = Logger::new(LOG_LEVEL);

const MAX_NUM_NODES: usize = sel4_config::sel4_cfg_usize!(MAX_NUM_NODES);
const NUM_SECONDARY_CORES: usize = MAX_NUM_NODES - 1;

static KERNEL_ENTRY_BARRIER: Barrier = Barrier::new(MAX_NUM_NODES);

pub fn main<'a>(payload: &Payload<'a>, own_footprint: &Range<usize>) -> ! {
    debug::init();

    LOGGER.set().unwrap();

    log::info!("Starting loader");

    log::debug!("Platform info: {:#x?}", PLATFORM_INFO);
    log::debug!("Loader footprint: {:#x?}", own_footprint);
    log::debug!("Payload info: {:#x?}", payload.info);
    log::debug!("Payload regions:");
    for content in payload.data.iter() {
        log::debug!(
            "    0x{:x?} {:?}",
            content.phys_addr_range,
            content.content.is_some()
        );
    }

    {
        let own_footprint =
            own_footprint.start.try_into().unwrap()..own_footprint.end.try_into().unwrap();
        loader_sanity_check::sanity_check(&own_footprint, &payload.data);
    }

    log::debug!("Copying payload data");
    copy_payload_data::copy_payload_data(&payload.data);

    log::debug!("Initializing translation structures");
    {
        let kernel_phys_start = payload.info.kernel_image.phys_addr_range.start;
        let kernel_virt_start = payload.info.kernel_image.virt_addr_range().start;
        init_translation_structures::init_translation_structures(
            kernel_phys_start.try_into().unwrap(),
            kernel_virt_start.try_into().unwrap(),
        );
    }

    smp::start_secondary_cores(&payload.info);

    common_epilogue(0, &payload.info)
}

fn secondary_core_main(core_id: usize, payload_info: &PayloadInfo) -> ! {
    common_epilogue(core_id, payload_info)
}

fn common_epilogue(core_id: usize, payload_info: &PayloadInfo) -> ! {
    if core_id == 0 {
        log::info!("Entering kernel");
    }
    KERNEL_ENTRY_BARRIER.wait();
    init_platform_state::init_platform_state_per_core(core_id);
    log::debug!("Core {}: Entering kernel", core_id);
    init_platform_state::init_platform_state_per_core_after_which_no_syncronization(core_id);
    enter_kernel::enter_kernel(&payload_info);
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
        wfe();
    }
}
