//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use sel4_microkit::{
    protection_domain, Channel, DeferredAction, DeferredActionSlot, Handler, Infallible,
};

const CLIENT: Channel = Channel::new(0);

#[protection_domain]
fn init() -> impl Handler {
    HandlerImpl {
        deferred_action: DeferredActionSlot::new(),
    }
}

struct HandlerImpl {
    deferred_action: DeferredActionSlot,
}

impl Handler for HandlerImpl {
    type Error = Infallible;

    fn notified(&mut self, _channel: Channel) -> Result<(), Self::Error> {
        self.deferred_action.defer_notify(CLIENT).unwrap();
        Ok(())
    }

    fn take_deferred_action(&mut self) -> Option<DeferredAction> {
        self.deferred_action.take()
    }
}
