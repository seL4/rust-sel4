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

use core::ptr::NonNull;

use virtio_drivers::{
    device::blk::*,
    device::net::*,
    transport::{
        mmio::{MmioTransport, VirtIOHeader},
        DeviceType, Transport,
    },
};

use sel4_logging::{LevelFilter, Logger, LoggerBuilder};
use sel4cp::{memory_region_symbol, protection_domain, var, Channel, Handler};
use tests_capdl_http_server_components_http_server_core::run_server;
use tests_capdl_http_server_components_sp804_driver_core::Driver;

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

const NET_BUFFER_LEN: usize = 2048;

const TIMER_DEVICE: Channel = Channel::new(0);
const BLK_DEVICE: Channel = Channel::new(1);
const NET_DEVICE: Channel = Channel::new(2);

#[protection_domain(
    heap_size = 16 * 1024 * 1024,
)]
fn init() -> impl Handler {
    LOGGER.set().unwrap();

    setup_newlib();

    let timer = unsafe {
        Driver::new(
            memory_region_symbol!(sp804_mmio_vaddr: *mut ()).as_ptr(),
            var!(timer_freq: usize = 0).clone().try_into().unwrap(),
        )
    };

    HalImpl::init(
        *var!(virtio_dma_size: usize = 0),
        *var!(virtio_dma_vaddr: usize = 0),
        *var!(virtio_dma_paddr: usize = 0),
    );

    let virtio_mmio_vaddr = var!(virtio_mmio_vaddr: usize = 0);
    let virtio_net_mmio_offset = var!(virtio_net_mmio_offset: usize = 0);

    let net_device = {
        let header =
            NonNull::new((virtio_mmio_vaddr + virtio_net_mmio_offset) as *mut VirtIOHeader)
                .unwrap();
        let transport = unsafe { MmioTransport::new(header) }.unwrap();
        assert_eq!(transport.device_type(), DeviceType::Network);
        DeviceImpl::new(VirtIONet::new(transport, NET_BUFFER_LEN).unwrap())
    };

    let blk_device = {
        let header = NonNull::new(
            (virtio_mmio_vaddr + var!(virtio_blk_mmio_offset: usize = 0)) as *mut VirtIOHeader,
        )
        .unwrap();
        let transport = unsafe { MmioTransport::new(header) }.unwrap();
        assert_eq!(transport.device_type(), DeviceType::Block);
        CpiofsBlockIOImpl::new(VirtIOBlk::new(transport).unwrap())
    };

    Reactor::new(
        net_device,
        blk_device,
        timer,
        NET_DEVICE,
        BLK_DEVICE,
        TIMER_DEVICE,
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
