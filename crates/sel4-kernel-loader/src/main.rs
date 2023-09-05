#![no_std]
#![no_main]
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

use core::panic::PanicInfo;

use sel4_logging::LevelFilter;

mod arch;
mod barrier;
mod copy_payload_data;
mod drivers;
mod enter_kernel;
mod fmt;
mod logging;
mod plat;
mod run;
mod sanity_check;
mod smp;
mod stacks;
mod this_image;

const LOG_LEVEL: LevelFilter = LevelFilter::Debug;

const MAX_NUM_NODES: usize = sel4_config::sel4_cfg_usize!(MAX_NUM_NODES);

const NUM_SECONDARY_CORES: usize = MAX_NUM_NODES - 1;

#[no_mangle]
extern "C" fn main() -> ! {
    run::run(
        this_image::get_payload,
        &this_image::get_user_image_bounds(),
    )
}

#[panic_handler]
extern "C" fn panic_handler(info: &PanicInfo) -> ! {
    log::error!("{}", info);
    arch::idle()
}

mod translation_tables {
    #[sel4_config::sel4_cfg(ARCH_AARCH64)]
    mod loader {
        include!(concat!(env!("OUT_DIR"), "/loader_translation_tables.rs"));
    }
    mod kernel {
        include!(concat!(env!("OUT_DIR"), "/kernel_translation_tables.rs"));
    }
}
