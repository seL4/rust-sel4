#![no_std]
#![no_main]
#![feature(never_type)]

use sel4cp::{protection_domain, Channel, DeferredAction, DeferredActionSlot, Handler};

const CLIENT: Channel = Channel::new(0);

#[protection_domain]
fn init() -> impl Handler {
    ThisHandler {
        deferred_action: DeferredActionSlot::new(),
    }
}

struct ThisHandler {
    deferred_action: DeferredActionSlot,
}

impl Handler for ThisHandler {
    type Error = !;

    fn notified(&mut self, _channel: Channel) -> Result<(), Self::Error> {
        self.deferred_action.defer(CLIENT.defer_notify()).unwrap();
        Ok(())
    }

    fn take_deferred_action(&mut self) -> Option<DeferredAction> {
        self.deferred_action.take()
    }
}
