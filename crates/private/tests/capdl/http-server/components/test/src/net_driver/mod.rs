use alloc::vec::Vec;
use core::alloc::{GlobalAlloc, Layout};
use core::cell::RefCell;
use core::ops::Range;
use core::ptr::{self, NonNull};
use core::slice;

use log::{debug, info, trace, warn};
use serde::{Deserialize, Serialize};
use virtio_drivers::{
    device::net::*,
    transport::{
        mmio::{MmioTransport, VirtIOHeader},
        DeviceType, Transport,
    },
    BufferDirection, Hal,
};

use sel4_simple_task_runtime::{debug_print, debug_println};

use crate::Config;

mod hal;

use hal::HalImpl;

const MMIO_REGION_SIZE: usize = 0x200;

const NET_BUFFER_LEN: usize = 2048;
const NET_QUEUE_SIZE: usize = 16;

pub fn test_driver(config: &Config) {
    HalImpl::init(
        NonNull::slice_from_raw_parts(
            NonNull::new(config.virtio_net_dma_vaddr_range.start as *mut _).unwrap(),
            config.virtio_net_dma_vaddr_range.end - config.virtio_net_dma_vaddr_range.start,
        ),
        config.virtio_net_dma_vaddr_to_paddr_offset,
    );

    let mut net: VirtIONet<HalImpl, MmioTransport, NET_QUEUE_SIZE> = {
        let header = NonNull::new(
            (config.virtio_net_mmio_vaddr + config.virtio_net_mmio_offset) as *mut VirtIOHeader,
        )
        .unwrap();
        let transport = unsafe { MmioTransport::new(header) }.unwrap();
        assert_eq!(transport.device_type(), DeviceType::Network);
        VirtIONet::new(transport, NET_BUFFER_LEN).unwrap()
    };

    let event_nfn = config.event_nfn.get();
    loop {
        let (_, _badge) = event_nfn.wait();

        while net.can_recv() {
            let buf = net.receive().unwrap();
            let packet = buf.packet();
            debug_println!("packet:");
            for b in packet.iter() {
                debug_print!("{:02X} ", b);
            }
            debug_println!();
            net.recycle_rx_buffer(buf).unwrap();
        }

        config
            .virtio_net_irq_handler
            .get()
            .irq_handler_ack()
            .unwrap();

        net.ack_interrupt();
    }
}
