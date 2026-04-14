//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_microkit::{Channel, ChannelSet, DeferredAction, DeferredActionSlot, Handler, Infallible};

const CLIENT: Channel = Channel::new(0);

pub(crate) fn init() -> HandlerImpl {
    HandlerImpl {
        deferred_action: DeferredActionSlot::new(),
    }
}

pub(crate) struct HandlerImpl {
    deferred_action: DeferredActionSlot,
}

impl Handler for HandlerImpl {
    type Error = Infallible;

    fn notified(&mut self, _channels: ChannelSet) -> Result<(), Self::Error> {
        self.deferred_action.defer_notify(CLIENT).unwrap();
        Ok(())
    }

    fn take_deferred_action(&mut self) -> Option<DeferredAction> {
        self.deferred_action.take()
    }
}
