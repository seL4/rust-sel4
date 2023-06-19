#![no_std]
#![no_main]
#![feature(error_in_core)]
#![feature(allocator_api)]
#![feature(thread_local)]
#![feature(btreemap_alloc)]
#![feature(lazy_cell)]
#![feature(strict_provenance)]
#![feature(slice_ptr_get)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

extern crate alloc;

use alloc::vec::Vec;
use core::alloc::{GlobalAlloc, Layout};
use core::cell::RefCell;
use core::ops::Range;
use core::ptr::{self, NonNull};
use core::slice;

use log::{debug, info, trace, warn};
use serde::{Deserialize, Serialize};

use sel4_logging::{LevelFilter, Logger, LoggerBuilder};
use sel4_simple_task_config_types::*;
use sel4_simple_task_runtime::{debug_print, debug_println, main_json};

mod net_driver;
mod timer;

const LOG_LEVEL: LevelFilter = LevelFilter::Trace;

static LOGGER: Logger = LoggerBuilder::const_default()
    .level_filter(LOG_LEVEL)
    .filter(|meta| !meta.target().starts_with("sel4_sys"))
    .write(|s| sel4::debug_print!("{}", s))
    .build();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub event_nfn: ConfigCPtr<Notification>,
    pub virtio_irq_range: Range<usize>,
    pub virtio_irq_handlers: Vec<ConfigCPtr<IRQHandler>>,
    pub virtio_mmio_vaddr_range: Range<usize>,
    pub virtio_dma_vaddr_range: Range<usize>,
    pub virtio_dma_vaddr_to_paddr_offset: isize,
    pub timer_irq_handler: ConfigCPtr<IRQHandler>,
    pub timer_mmio_vaddr: usize,
    pub timer_freq: usize,
}

#[main_json]
fn main(config: Config) {
    LOGGER.set().unwrap();

    // debug_println!("{:#x?}", config);

    net_driver::test_driver(&config);
    // timer::test_driver(&config);

    debug_println!("TEST_PASS");
}
