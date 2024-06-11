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

use super::common::*;

/// Device-independent embedded_hal_nb::serial interface to a serial-device
/// component. Interact with it using [serial::Read], [serial::Write],
/// and [fmt::Write].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Client {
    pub channel: Channel,
}

impl Client {
    pub fn new(channel: Channel) -> Self {
        Client { channel }
    }

    pub fn blocking_write(&mut self, val: u8) -> Result<(), Error> {
        let req = Request::PutChar { val };
        self.channel
            .pp_call(MessageInfo::send_using_postcard(req).unwrap())
            .recv_using_postcard::<Result<PutCharResponse, PutCharError>>()
            .map_err(|_| Error::InvalidResponse)?
            .map_err(Error::PutCharError)?;
        Ok(())
    }
}

impl serial::ErrorType for Client {
    type Error = Error;
}

impl serial::Read<u8> for Client {
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        let req = Request::GetChar;
        let resp = self
            .channel
            .pp_call(MessageInfo::send_using_postcard(req).unwrap())
            .recv_using_postcard::<Result<GetCharResponse, GetCharError>>()
            .map_err(|_| Error::InvalidResponse)
            .map_err(nb::Error::Other)?
            .map_err(Error::GetCharError)
            .map_err(nb::Error::Other)?;
        resp.val.ok_or(nb::Error::WouldBlock)
    }
}

impl serial::Write<u8> for Client {
    fn write(&mut self, val: u8) -> nb::Result<(), Self::Error> {
        self.blocking_write(val).map_err(nb::Error::Other)?;
        Ok(())
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Error {
    PutCharError(PutCharError),
    GetCharError(GetCharError),
    InvalidResponse,
}

impl serial::Error for Error {
    fn kind(&self) -> serial::ErrorKind {
        serial::ErrorKind::Other
    }
}
