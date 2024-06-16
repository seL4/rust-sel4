//
// Copyright 2023, Colias Group, LLC
// Copyright 2023, Galois, Inc.
//
// SPDX-License-Identifier: BSD-2-Clause
//

use embedded_hal_nb::nb;
use embedded_hal_nb::serial;

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

impl serial::ErrorType for Client {
    type Error = Error;
}

impl serial::Read<u8> for Client {
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        match self.request(Request::Read)? {
            SuccessResponse::Read(v) => v.into(),
            _ => Err(Error::UnexpectedResponse.into()),
        }
    }
}

impl serial::Write<u8> for Client {
    fn write(&mut self, v: u8) -> nb::Result<(), Self::Error> {
        match self.request(Request::Write(v))? {
            SuccessResponse::Write(v) => v.into(),
            _ => Err(Error::UnexpectedResponse.into()),
        }
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        match self.request(Request::Flush)? {
            SuccessResponse::Flush(v) => v.into(),
            _ => Err(Error::UnexpectedResponse.into()),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Error {
    ErrorResponse(ErrorResponse),
    InvalidResponse,
    UnexpectedResponse,
}

impl serial::Error for Error {
    fn kind(&self) -> serial::ErrorKind {
        serial::ErrorKind::Other
    }
}
