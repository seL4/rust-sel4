//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::convert::Infallible;

use rtcc::DateTimeAccess;

use sel4_microkit::{Channel, Handler, MessageInfo};
use sel4_microkit_simple_ipc as simple_ipc;

use super::message_types::*;

/// Handle messages using an implementor of [serial::Read<u8>] and [serial::Write<u8>].
#[derive(Clone, Debug)]
pub struct HandlerImpl<Driver> {
    driver: Driver,
    client: Channel,
}

impl<Driver> HandlerImpl<Driver> {
    pub fn new(driver: Driver, client: Channel) -> Self {
        Self { driver, client }
    }
}

impl<Driver> Handler for HandlerImpl<Driver>
where
    Driver: DateTimeAccess,
{
    type Error = Infallible;

    fn protected(
        &mut self,
        channel: Channel,
        msg_info: MessageInfo,
    ) -> Result<MessageInfo, Self::Error> {
        if channel == self.client {
            Ok(match simple_ipc::recv::<Request>(msg_info) {
                Ok(req) => {
                    let resp = match req {
                        Request::DateTime => self
                            .driver
                            .datetime()
                            .map(SuccessResponse::DateTime)
                            .map_err(|_| ErrorResponse::DateTimeError),
                        Request::SetDateTime(v) => self
                            .driver
                            .set_datetime(&v)
                            .map(|_| SuccessResponse::SetDateTime)
                            .map_err(|_| ErrorResponse::SetDateTimeError),
                    };
                    simple_ipc::send(resp)
                }
                Err(_) => simple_ipc::send_unspecified_error(),
            })
        } else {
            panic!("unexpected channel: {channel:?}");
        }
    }
}
