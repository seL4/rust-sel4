#![no_std]
#![no_main]
#![feature(async_fn_in_trait)]
#![feature(int_roundings)]
#![feature(never_type)]
#![feature(pattern)]
#![feature(ptr_metadata)]
#![feature(slice_ptr_get)]
#![feature(strict_provenance)]
#![feature(try_blocks)]

extern crate alloc;

use core::ops::Range;
use core::ptr::NonNull;

use serde::{Deserialize, Serialize};

use virtio_drivers::{
    device::blk::*,
    device::net::*,
    transport::{
        mmio::{MmioTransport, VirtIOHeader},
        DeviceType, Transport,
    },
};

use sel4_logging::{LevelFilter, Logger, LoggerBuilder};
use sel4_simple_task_config_types::*;
use sel4_simple_task_runtime::main_json;
use tests_capdl_http_server_components_test_server_core::run_server;
use tests_capdl_http_server_components_test_sp804_driver::Driver;

mod glue;
mod reactor;

use glue::{CpiofsBlockIOImpl, DeviceImpl, HalImpl, BLOCK_SIZE};
use reactor::Reactor;

const CERT_PEM: &str = concat!(include_str!(concat!(env!("OUT_DIR"), "/cert.pem")), "\0");
const PRIV_PEM: &str = concat!(include_str!(concat!(env!("OUT_DIR"), "/priv.pem")), "\0");

// const LOG_LEVEL: LevelFilter = LevelFilter::Trace;
// const LOG_LEVEL: LevelFilter = LevelFilter::Debug;
const LOG_LEVEL: LevelFilter = LevelFilter::Info;
// const LOG_LEVEL: LevelFilter = LevelFilter::Warn;

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
    pub virtio_blk_irq_handler: ConfigCPtr<IRQHandler>,
    pub virtio_blk_mmio_vaddr: usize,
    pub virtio_blk_mmio_offset: usize,
    pub virtio_dma_vaddr_range: Range<usize>,
    pub virtio_dma_paddr: usize,
}

const NET_BUFFER_LEN: usize = 2048;

#[main_json]
fn main(config: Config) -> ! {
    LOGGER.set().unwrap();

    setup_newlib();

    let timer = unsafe {
        Driver::new(
            config.timer_mmio_vaddr as *mut (),
            config.timer_freq.try_into().unwrap(),
        )
    };

    HalImpl::init(
        config.virtio_dma_vaddr_range.len(),
        config.virtio_dma_vaddr_range.start,
        config.virtio_dma_paddr,
    );

    let net_device = {
        let header = NonNull::new(
            (config.virtio_net_mmio_vaddr + config.virtio_net_mmio_offset) as *mut VirtIOHeader,
        )
        .unwrap();
        let transport = unsafe { MmioTransport::new(header) }.unwrap();
        assert_eq!(transport.device_type(), DeviceType::Network);
        DeviceImpl::new(VirtIONet::new(transport, NET_BUFFER_LEN).unwrap())
    };

    let blk_device = {
        let header = NonNull::new(
            (config.virtio_blk_mmio_vaddr + config.virtio_blk_mmio_offset) as *mut VirtIOHeader,
        )
        .unwrap();
        let transport = unsafe { MmioTransport::new(header) }.unwrap();
        assert_eq!(transport.device_type(), DeviceType::Block);
        CpiofsBlockIOImpl::new(VirtIOBlk::new(transport).unwrap())
    };

    let reactor = Reactor::new(
        net_device,
        blk_device,
        timer,
        config.virtio_net_irq_handler.get(),
        config.virtio_blk_irq_handler.get(),
        config.timer_irq_handler.get(),
    );

    reactor.run(
        config.event_nfn.get(),
        |network_ctx, timers_ctx, blk_device, spawner| {
            run_server(
                network_ctx,
                timers_ctx,
                blk_device,
                spawner,
                CERT_PEM,
                PRIV_PEM,
            )
        },
    )
}

fn setup_newlib() {
    use sel4_newlib::*;

    set_static_heap_for_sbrk({
        static HEAP: StaticHeap<{ 1024 * 1024 }> = StaticHeap::new();
        &HEAP
    });

    let mut impls = Implementations::default();
    impls._sbrk = Some(sbrk_with_static_heap);
    impls._write = Some(write_with_debug_put_char);
    set_implementations(impls)
}
