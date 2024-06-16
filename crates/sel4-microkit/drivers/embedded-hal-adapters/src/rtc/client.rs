//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use rtcc::{DateTimeAccess, NaiveDateTime};

use sel4_microkit::{Channel, MessageInfo};
use sel4_microkit_message::MessageInfoExt;

use super::message_types::*;

/// Device-independent embedded_hal_nb::serial interface to a serial-device
/// component. Interact with it using [serial::Read], [serial::Write],
/// and [fmt::Write].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Client {
    channel: Channel,
}

impl Client {
    pub fn new(channel: Channel) -> Self {
        Client { channel }
    }

    fn request(&self, req: Request) -> Result<SuccessResponse, Error> {
        self.channel
            .pp_call(MessageInfo::send_using_postcard(req).unwrap())
            .recv_using_postcard::<Response>()
            .map_err(|_| Error::InvalidResponse)?
            .map_err(Error::ErrorResponse)
    }
}

impl DateTimeAccess for Client {
    type Error = Error;

    fn datetime(&mut self) -> Result<NaiveDateTime, Self::Error> {
        match self.request(Request::DateTime)? {
            SuccessResponse::DateTime(v) => Ok(v),
            _ => Err(Error::UnexpectedResponse),
        }
    }

    fn set_datetime(&mut self, v: &NaiveDateTime) -> Result<(), Self::Error> {
        match self.request(Request::SetDateTime(*v))? {
            SuccessResponse::SetDateTime => Ok(()),
            _ => Err(Error::UnexpectedResponse),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Error {
    ErrorResponse(ErrorResponse),
    InvalidResponse,
    UnexpectedResponse,
}
