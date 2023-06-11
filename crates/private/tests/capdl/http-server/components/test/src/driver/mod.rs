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
        config.dma_vaddr_range.start as *mut u8..config.dma_vaddr_range.end as *mut u8,
        config.dma_vaddr_to_paddr_offset,
    );

    let mut net = None;

    for region in config.mmio_vaddr_range.clone().step_by(MMIO_REGION_SIZE) {
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
                    .replace(VirtIONetRaw::new(transport).expect("failed to create net driver"))
                    .is_none())
            }
        }
    }

    let mut net: VirtIONetRaw<HalImpl, MmioTransport, NET_QUEUE_SIZE> = net.unwrap();

    let event_nfn = config.event_nfn.get();
    loop {
        // let (_, badge) = event_nfn.wait();
        // info!("badge: {:x}", badge);

        // while net.can_receive() {
        unsafe {
            let (paddr, vaddr) = HalImpl::dma_alloc(1, BufferDirection::Both);
            let buf = slice::from_raw_parts_mut(vaddr.as_ptr(), 4096);
            let (n_header, n_packet) = net.receive_wait(buf).unwrap();
            debug_println!("n_header: {n_header}, n_packet: {n_packet}");
            // debug_println!("packet: {:02X?}", &buf[n_header..][..n_packet]);
            for b in &buf[n_header..][..n_packet] {
                debug_print!("{:02X} ", b);
            }
            debug_println!();
        }
        // let buf = net.receive().unwrap();
        // let packet = buf.packet();
        // debug_println!("packet: {:X?}", packet);
        // }

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
