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

use smoltcp::iface::Config;
use smoltcp::phy::{Device, Medium};
use smoltcp::wire::{EthernetAddress, HardwareAddress};

use sel4_externally_shared::ExternallySharedRef;
use sel4_logging::{LevelFilter, Logger, LoggerBuilder};
use sel4_shared_ring_buffer::{RingBuffer, RingBuffers};
use sel4_shared_ring_buffer_block_io::{BlockIO, BLOCK_SIZE};
use sel4_shared_ring_buffer_smoltcp::DeviceImpl;
use sel4cp::{memory_region_symbol, protection_domain, var, Channel, Handler};

use sel4cp_http_server_example_server_core::run_server;

mod handler;
mod net_client;
mod timer_client;

use handler::HandlerImpl;
use net_client::NetClient;
use timer_client::TimerClient;

const CERT_PEM: &str = concat!(include_str!(concat!(env!("OUT_DIR"), "/cert.pem")), "\0");
const PRIV_PEM: &str = concat!(include_str!(concat!(env!("OUT_DIR"), "/priv.pem")), "\0");

const LOG_LEVEL: LevelFilter = {
    // LevelFilter::Trace
    // LevelFilter::Debug
    LevelFilter::Info
    // LevelFilter::Warn
};

static LOGGER: Logger = LoggerBuilder::const_default()
    .level_filter(LOG_LEVEL)
    .filter(|meta| !meta.target().starts_with("sel4_sys"))
    .write(|s| sel4::debug_print!("{}", s))
    .build();

const TIMER_DRIVER: Channel = Channel::new(0);
const NET_DRIVER: Channel = Channel::new(1);
const BLOCK_DRIVER: Channel = Channel::new(2);

#[protection_domain(
    heap_size = 16 * 1024 * 1024,
)]
fn init() -> impl Handler {
    LOGGER.set().unwrap();

    setup_newlib();

    let timer_client = TimerClient::new(TIMER_DRIVER);
    let net_client = NetClient::new(NET_DRIVER);

    let notify_net = || {
        NET_DRIVER.notify();
        Ok::<_, !>(())
    };

    let notify_block = || {
        BLOCK_DRIVER.notify();
        Ok::<_, !>(())
    };

    let net_device = DeviceImpl::new(
        unsafe {
            ExternallySharedRef::<'static, _>::new(
                memory_region_symbol!(virtio_net_client_dma_vaddr: *mut [u8], n = *var!(virtio_net_client_dma_size: usize = 0)),
            )
        },
        *var!(virtio_net_client_dma_paddr: usize = 0),
        unsafe {
            RingBuffers::new(
                RingBuffer::from_ptr(memory_region_symbol!(virtio_net_rx_free: *mut _)),
                RingBuffer::from_ptr(memory_region_symbol!(virtio_net_rx_used: *mut _)),
                notify_net,
                true,
            )
        },
        unsafe {
            RingBuffers::new(
                RingBuffer::from_ptr(memory_region_symbol!(virtio_net_tx_free: *mut _)),
                RingBuffer::from_ptr(memory_region_symbol!(virtio_net_tx_used: *mut _)),
                notify_net,
                true,
            )
        },
        16,
        2048,
        1500,
    );

    let net_config = {
        assert_eq!(net_device.capabilities().medium, Medium::Ethernet);
        let mac_address = EthernetAddress(net_client.get_mac_address().0);
        let hardware_addr = HardwareAddress::Ethernet(mac_address);
        let mut this = Config::new(hardware_addr);
        this.random_seed = 0;
        this
    };

    let fs_block_io = BlockIO::new(
        unsafe {
            ExternallySharedRef::<'static, _>::new(
                memory_region_symbol!(virtio_blk_client_dma_vaddr: *mut [u8], n = *var!(virtio_blk_client_dma_size: usize = 0)),
            )
        },
        *var!(virtio_blk_client_dma_paddr: usize = 0),
        unsafe {
            RingBuffers::new(
                RingBuffer::from_ptr(memory_region_symbol!(virtio_blk_free: *mut _)),
                RingBuffer::from_ptr(memory_region_symbol!(virtio_blk_used: *mut _)),
                notify_block,
                true,
            )
        },
    );

    HandlerImpl::new(
        TIMER_DRIVER,
        NET_DRIVER,
        BLOCK_DRIVER,
        timer_client,
        net_device,
        net_config,
        fs_block_io,
        |timers_ctx, network_ctx, fs_io, spawner| {
            run_server(timers_ctx, network_ctx, fs_io, spawner, CERT_PEM, PRIV_PEM)
        },
    )
}

fn setup_newlib() {
    use sel4_newlib::*;

    set_static_heap_for_sbrk({
        static HEAP: StaticHeap<{ 1024 * 1024 }> = StaticHeap::new();
        &HEAP
    });

    set_implementations(Implementations {
        _sbrk: Some(sbrk_with_static_heap),
        _write: Some(write_with_debug_put_char),
        ..Default::default()
    })
}
