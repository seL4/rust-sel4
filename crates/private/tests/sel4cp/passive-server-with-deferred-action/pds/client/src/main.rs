#![no_std]
#![no_main]
#![feature(never_type)]

use sel4cp::{debug_println, protection_domain, Channel, Handler};

const SERVER: Channel = Channel::new(0);

#[protection_domain]
fn init() -> impl Handler {
    SERVER.notify();
    ThisHandler {}
}

struct ThisHandler {}

impl Handler for ThisHandler {
    type Error = !;

    fn notified(&mut self, _channel: Channel) -> Result<(), Self::Error> {
        debug_println!("TEST_PASS");
        Ok(())
    }
}
