//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

extern crate alloc;

use alloc::rc::Rc;
use alloc::sync::Arc;
use core::time::Duration;

use smoltcp::iface::Config;
use smoltcp::phy::{Device, Medium};
use smoltcp::wire::{EthernetAddress, HardwareAddress};

use sel4_async_block_io::{
    constant_block_sizes::BlockSize512, disk::Disk, CachedBlockIO, ConstantBlockSize,
};
use sel4_async_time::Instant;
use sel4_bounce_buffer_allocator::{Basic, BounceBufferAllocator};
use sel4_externally_shared::{ExternallySharedRef, ExternallySharedRefExt};
use sel4_logging::{LevelFilter, Logger, LoggerBuilder};
use sel4_microkit::{memory_region_symbol, protection_domain, Handler};
use sel4_newlib as _;
use sel4_shared_ring_buffer::RingBuffers;
use sel4_shared_ring_buffer_block_io::SharedRingBufferBlockIO;
use sel4_shared_ring_buffer_smoltcp::DeviceImpl;

use microkit_http_server_example_server_core::run_server;

mod block_client;
mod config;
mod handler;
mod net_client;
mod rtc_client;
mod timer_client;

use block_client::BlockClient;
use config::channels;
use handler::HandlerImpl;
use net_client::NetClient;
use rtc_client::RtcClient;
use timer_client::TimerClient;

const BLOCK_CACHE_SIZE_IN_BLOCKS: usize = 128;

const MAX_NUM_SIMULTANEOUS_CONNECTIONS: usize = 32;

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

#[protection_domain(
    heap_size = 16 * 1024 * 1024,
)]
fn init() -> impl Handler {
    LOGGER.set().unwrap();

    let rtc_client = RtcClient::new(channels::RTC_DRIVER);
    let timer_client = Arc::new(TimerClient::new(channels::TIMER_DRIVER));
    let net_client = NetClient::new(channels::NET_DRIVER);
    let block_client = BlockClient::new(channels::BLOCK_DRIVER);

    let now_unix_time = Duration::from_secs(rtc_client.now().into());

    let now_fn = {
        let timer_client = timer_client.clone();
        move || Instant::ZERO + Duration::from_micros(timer_client.now())
    };

    let notify_net: fn() = || channels::NET_DRIVER.notify();
    let notify_block: fn() = || channels::BLOCK_DRIVER.notify();

    let net_device = {
        let dma_region = unsafe {
            ExternallySharedRef::<'static, _>::new(
                memory_region_symbol!(virtio_net_client_dma_vaddr: *mut [u8], n = config::VIRTIO_NET_CLIENT_DMA_SIZE),
            )
        };

        let bounce_buffer_allocator =
            BounceBufferAllocator::new(Basic::new(dma_region.as_ptr().len()), 1);

        DeviceImpl::new(
            dma_region,
            bounce_buffer_allocator,
            RingBuffers::from_ptrs_using_default_initialization_strategy_for_role(
                unsafe {
                    ExternallySharedRef::new(memory_region_symbol!(virtio_net_rx_free: *mut _))
                },
                unsafe {
                    ExternallySharedRef::new(memory_region_symbol!(virtio_net_rx_used: *mut _))
                },
                notify_net,
            ),
            RingBuffers::from_ptrs_using_default_initialization_strategy_for_role(
                unsafe {
                    ExternallySharedRef::new(memory_region_symbol!(virtio_net_tx_free: *mut _))
                },
                unsafe {
                    ExternallySharedRef::new(memory_region_symbol!(virtio_net_tx_used: *mut _))
                },
                notify_net,
            ),
            16,
            2048,
            1500,
        )
        .unwrap()
    };

    let net_config = {
        assert_eq!(net_device.capabilities().medium, Medium::Ethernet);
        let mac_address = EthernetAddress(net_client.get_mac_address().0);
        let hardware_addr = HardwareAddress::Ethernet(mac_address);
        let mut this = Config::new(hardware_addr);
        this.random_seed = 0;
        this
    };

    let num_blocks = block_client.get_num_blocks();

    let shared_block_io = {
        let dma_region = unsafe {
            ExternallySharedRef::<'static, _>::new(
                memory_region_symbol!(virtio_blk_client_dma_vaddr: *mut [u8], n = config::VIRTIO_BLK_CLIENT_DMA_SIZE),
            )
        };

        let bounce_buffer_allocator =
            BounceBufferAllocator::new(Basic::new(dma_region.as_ptr().len()), 1);

        SharedRingBufferBlockIO::new(
            BlockSize512::BLOCK_SIZE,
            num_blocks,
            dma_region,
            bounce_buffer_allocator,
            RingBuffers::from_ptrs_using_default_initialization_strategy_for_role(
                unsafe { ExternallySharedRef::new(memory_region_symbol!(virtio_blk_free: *mut _)) },
                unsafe { ExternallySharedRef::new(memory_region_symbol!(virtio_blk_used: *mut _)) },
                notify_block,
            ),
        )
    };

    HandlerImpl::new(
        channels::TIMER_DRIVER,
        channels::NET_DRIVER,
        channels::BLOCK_DRIVER,
        timer_client,
        net_device,
        net_config,
        shared_block_io.clone(),
        |timers_ctx, network_ctx, spawner| async move {
            let fs_block_io = shared_block_io.clone();
            let fs_block_io = CachedBlockIO::new(fs_block_io.clone(), BLOCK_CACHE_SIZE_IN_BLOCKS);
            let disk = Disk::new(fs_block_io);
            let entry = disk.read_mbr().await.unwrap().partition(0).unwrap();
            let fs_block_io = disk.partition_using_mbr(&entry);
            let fs_block_io = Rc::new(fs_block_io);
            run_server(
                now_unix_time,
                now_fn,
                timers_ctx,
                network_ctx,
                fs_block_io,
                spawner,
                CERT_PEM,
                PRIV_PEM,
                MAX_NUM_SIMULTANEOUS_CONNECTIONS,
            )
            .await
        },
    )
}
