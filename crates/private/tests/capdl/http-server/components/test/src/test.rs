#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use alloc::vec::Vec;
use core::alloc::{GlobalAlloc, Layout};
use core::cell::RefCell;
use core::ops::Range;
use core::ptr::{self, NonNull};
use core::slice;
use core::time::Duration;

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

use sel4_logging::{LevelFilter, Logger, LoggerBuilder};
use sel4_simple_task_config_types::*;
use sel4_simple_task_runtime::{debug_print, debug_println, main_json};

use tests_capdl_http_server_components_test_sp804_driver::Driver as TimerDriver;

const TIMER_IRQ_BADGE: sel4::Badge = 1 << 0;
const VIRTIO_NET_IRQ_BADGE: sel4::Badge = 1 << 1;

pub fn test<const QUEUE_SIZE: usize>(
    event_nfn: self::Notification,
    timer: &mut TimerDriver,
    timer_irq_handler: sel4::IRQHandler,
    net: &mut VirtIONet<impl Hal, impl Transport, QUEUE_SIZE>,
    net_irq_handler: sel4::IRQHandler,
) {
    log::debug!("now: {:?}", timer.now());
    timer_irq_handler.irq_handler_ack().unwrap();
    timer.handle_interrupt();
    timer.set_timeout(Duration::from_secs(1));

    net_irq_handler.irq_handler_ack().unwrap();
    net.ack_interrupt();

    loop {
        let (_, badge) = event_nfn.wait();

        if badge & TIMER_IRQ_BADGE != 0 {
            log::debug!("now: {:?}", timer.now());
            timer_irq_handler.irq_handler_ack().unwrap();
            timer.handle_interrupt();
            timer.set_timeout(Duration::from_secs(1));
        }

        if badge & VIRTIO_NET_IRQ_BADGE != 0 {
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
            net_irq_handler.irq_handler_ack().unwrap();
            net.ack_interrupt();
        }
    }
}
