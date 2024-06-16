//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::convert::Infallible;

use rtcc::DateTimeAccess;

use sel4_microkit::{Channel, Handler, MessageInfo};
use sel4_microkit_message::MessageInfoExt;

use super::message_types::*;

/// Handle messages using an implementor of [serial::Read<u8>] and [serial::Write<u8>].
#[derive(Clone, Debug)]
pub struct Driver<Device> {
    device: Device,
    client: Channel,
}

impl<Device> Driver<Device> {
    pub fn new(device: Device, client: Channel) -> Self {
        Self { device, client }
    }
}

impl<Device> Handler for Driver<Device>
where
    Device: DateTimeAccess,
{
    type Error = Infallible;

    fn protected(
        &mut self,
        channel: Channel,
        msg_info: MessageInfo,
    ) -> Result<MessageInfo, Self::Error> {
        if channel == self.client {
            Ok(match msg_info.recv_using_postcard::<Request>() {
                Ok(req) => {
                    let resp = match req {
                        Request::DateTime => self
                            .device
                            .datetime()
                            .map(SuccessResponse::DateTime)
                            .map_err(|_| ErrorResponse::DateTimeError),
                        Request::SetDateTime(v) => self
                            .device
                            .set_datetime(&v)
                            .map(|_| SuccessResponse::SetDateTime)
                            .map_err(|_| ErrorResponse::SetDateTimeError),
                    };
                    MessageInfo::send_using_postcard(resp).unwrap()
                }
                Err(_) => MessageInfo::send_unspecified_error(),
            })
        } else {
            panic!("unexpected channel: {channel:?}");
        }
    }
}
