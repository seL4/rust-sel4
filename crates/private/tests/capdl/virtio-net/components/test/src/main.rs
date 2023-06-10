#![no_std]
#![no_main]
#![allow(unused_imports)]

extern crate alloc;

use alloc::vec::Vec;
use core::alloc::{GlobalAlloc, Layout};
use core::cell::RefCell;
use core::ops::Range;
use core::ptr::{self, NonNull};

use log::{debug, info, trace, warn};
use serde::{Deserialize, Serialize};
use virtio_drivers::{
    device::net::*,
    transport::{
        mmio::{MmioTransport, VirtIOHeader},
        DeviceType, Transport,
    },
};

use sel4_logging::{LevelFilter, Logger, LoggerBuilder};
use sel4_simple_task_config_types::*;
use sel4_simple_task_runtime::{debug_println, main_json};

mod hal;

use hal::HalImpl;

const LOG_LEVEL: LevelFilter = LevelFilter::Trace;

static LOGGER: Logger = LoggerBuilder::const_default()
    .level_filter(LOG_LEVEL)
    .write(|s| sel4::debug_print!("{}", s))
    .build();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub event_nfn: ConfigCPtr<Notification>,
    pub irq_range: Range<usize>,
    pub irq_handlers: Vec<ConfigCPtr<IRQHandler>>,
    pub mmio_vaddr_range: Range<usize>,
    pub dma_vaddr_range: Range<usize>,
    pub dma_vaddr_to_paddr_offset: isize,
}

const MMIO_REGION_SIZE: usize = 0x200;

const NET_BUFFER_LEN: usize = 2048;
const NET_QUEUE_SIZE: usize = 16;

#[main_json]
fn main(config: Config) {
    LOGGER.set().unwrap();

    debug_println!("{:#x?}", config);

    HalImpl::init(
        config.dma_vaddr_range.start as *mut u8 .. config.dma_vaddr_range.end as *mut u8,
        config.dma_vaddr_to_paddr_offset,
    );

    let mut net = None;

    for region in config.mmio_vaddr_range.step_by(MMIO_REGION_SIZE) {
        let header = NonNull::new(region as *mut VirtIOHeader).unwrap();
        match unsafe { MmioTransport::new(header) } {
            Err(e) => warn!("Error creating VirtIO MMIO transport: {}", e),
            Ok(transport) => {
                info!(
                    "Detected virtio MMIO device with vendor id {:#X}, device type {:?}, version {:?}",
                    transport.vendor_id(),
                    transport.device_type(),
                    transport.version(),
                );
                assert_eq!(transport.device_type(), DeviceType::Network);
                assert!(net
                    .replace(
                        VirtIONet::new(transport, NET_BUFFER_LEN)
                            .expect("failed to create net driver")
                    )
                    .is_none())
            }
        }
    }

    let mut net: VirtIONet<HalImpl, MmioTransport, NET_QUEUE_SIZE> = net.unwrap();

    let event_nfn = config.event_nfn.get();
    loop {
        let (_, badge) = event_nfn.wait();
        info!("badge: {:x}", badge);

        while net.can_recv() {
            let buf = net.receive().unwrap();
            let packet = buf.packet();
            debug_println!("packet: {:X?}", packet);
        }

        // for cap in config.irq_handlers.iter() {
        //     cap.get().irq_handler_ack().unwrap();
        // }

        // net.ack_interrupt();

        // unsafe {
        //     let p = 0x62000000 as *mut u64;
        //     *p = 0x13333337;
        // };
    }

    // debug_println!("TEST_PASS");
}
