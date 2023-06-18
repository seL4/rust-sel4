use alloc::vec::Vec;
use core::alloc::{GlobalAlloc, Layout};
use core::cell::RefCell;
use core::ops::Deref;
use core::ops::Range;
use core::ptr::{self, NonNull};
use core::slice;
use core::time::Duration;

use log::{debug, info, trace, warn};
use serde::{Deserialize, Serialize};
use tock_registers::interfaces::ReadWriteable;

use sel4_simple_task_runtime::{debug_print, debug_println};

use crate::Config;

mod device;
mod driver;

use driver::Driver;

pub fn test_driver(config: &Config) {
    let mut driver = unsafe {
        Driver::new(
            config.timer_mmio_vaddr as *mut (),
            config.timer_freq.try_into().unwrap(),
        )
    };
    let event_nfn = config.event_nfn.get();

    loop {
        config.timer_irq_handler.get().irq_handler_ack().unwrap();
        driver.handle_interrupt();
        debug!("now: {:?}", driver.now());
        driver.set_timeout(Duration::from_millis(10));
        let (_, _badge) = event_nfn.wait();
        debug!("x now: {:?}", driver.now());
        config.timer_irq_handler.get().irq_handler_ack().unwrap();
        driver.handle_interrupt();
    }

    let d = Duration::from_millis(10);

    let mut last = Duration::from_secs(0);

    loop {
        driver.handle_interrupt();
        let cur = driver.now();
        driver.handle_interrupt();
        // debug!("now: {:?}", cur);
        if cur - last > d {
            debug!("now: {:?}", cur);
            last = cur;
        }
    }
}
