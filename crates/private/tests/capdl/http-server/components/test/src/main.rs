#![no_std]
#![no_main]
#![feature(slice_ptr_get)]
#![feature(strict_provenance)]

extern crate alloc;

use core::ops::Range;

use serde::{Deserialize, Serialize};

use sel4_logging::{LevelFilter, Logger, LoggerBuilder};
use sel4_simple_task_config_types::*;
use sel4_simple_task_runtime::{debug_println, main_json};

mod net;
mod test;
mod timer;

// const LOG_LEVEL: LevelFilter = LevelFilter::Trace;
const LOG_LEVEL: LevelFilter = LevelFilter::Debug;

static LOGGER: Logger = LoggerBuilder::const_default()
    .level_filter(LOG_LEVEL)
    .filter(|meta| !meta.target().starts_with("sel4_sys"))
    .write(|s| sel4::debug_print!("{}", s))
    .build();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub event_nfn: ConfigCPtr<Notification>,
    pub timer_irq_handler: ConfigCPtr<IRQHandler>,
    pub timer_mmio_vaddr: usize,
    pub timer_freq: usize,
    pub virtio_net_irq_handler: ConfigCPtr<IRQHandler>,
    pub virtio_net_mmio_vaddr: usize,
    pub virtio_net_mmio_offset: usize,
    pub virtio_net_dma_vaddr_range: Range<usize>,
    pub virtio_net_dma_vaddr_to_paddr_offset: isize,
}

#[main_json]
fn main(config: Config) {
    LOGGER.set().unwrap();

    // debug_println!("{:#x?}", config);

    let mut timer = timer::init(&config);
    let mut net = net::init(&config);

    test::test(
        config.event_nfn.get(),
        &mut timer,
        config.timer_irq_handler.get(),
        &mut net,
        config.virtio_net_irq_handler.get(),
    );

    debug_println!("TEST_PASS");
}
