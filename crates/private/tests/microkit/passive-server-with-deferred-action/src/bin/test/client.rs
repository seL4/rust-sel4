//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_microkit::{Channel, ChannelSet, Handler, Infallible};

const SERVER: Channel = Channel::new(0);

pub(crate) fn init() -> HandlerImpl {
    SERVER.notify();
    HandlerImpl {}
}

pub(crate) struct HandlerImpl {}

impl Handler for HandlerImpl {
    type Error = Infallible;

    fn notified(&mut self, _channels: ChannelSet) -> Result<(), Self::Error> {
        sel4_test_microkit::indicate_success()
    }
}
