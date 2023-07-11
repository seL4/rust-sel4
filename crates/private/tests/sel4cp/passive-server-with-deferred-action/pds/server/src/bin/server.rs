#![no_std]
#![no_main]
#![feature(never_type)]

use sel4cp::{protection_domain, Channel, DeferredAction, Handler};

const CLIENT: Channel = Channel::new(0);

#[protection_domain]
fn init() -> impl Handler {
    ThisHandler {
        deferred_action: None,
    }
}

struct ThisHandler {
    deferred_action: Option<DeferredAction>,
}

impl Handler for ThisHandler {
    type Error = !;

    fn notified(&mut self, _channel: Channel) -> Result<(), Self::Error> {
        self.deferred_action = Some(CLIENT.defer_notify());
        Ok(())
    }

    fn take_deferred_action(&mut self) -> Option<DeferredAction> {
        self.deferred_action.take()
    }
}
