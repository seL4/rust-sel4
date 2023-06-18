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
            NonNull::new(config.virtio_dma_vaddr_range.start as *mut _).unwrap(),
            config.virtio_dma_vaddr_range.end - config.virtio_dma_vaddr_range.start,
        ),
        config.virtio_dma_vaddr_to_paddr_offset,
    );

    let mut net = None;

    for region in config
        .virtio_mmio_vaddr_range
        .clone()
        .step_by(MMIO_REGION_SIZE)
    {
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

        for cap in config.virtio_irq_handlers.iter() {
            cap.get().irq_handler_ack().unwrap();
        }

        net.ack_interrupt();
    }
}
