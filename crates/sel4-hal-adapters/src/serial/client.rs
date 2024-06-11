//
// Copyright 2023, Colias Group, LLC
// Copyright 2023, Galois, Inc.
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::fmt;

use embedded_hal_nb::nb;
use embedded_hal_nb::serial::{self, Write as _};

use sel4_microkit::{Channel, MessageInfo};
use sel4_microkit_message::MessageInfoExt;

use super::common::*;

/// Device-independent embedded_hal::serial interface to a serial-device
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
}

#[derive(Clone, Debug)]
pub enum Error {
    ReadError(IpcError),
    WriteError(IpcError),
}

#[derive(Clone, Debug)]
pub enum IpcError {
    GotError,
    GotInvalidResponse,
}

impl serial::Error for Error {
    fn kind(&self) -> serial::ErrorKind {
        serial::ErrorKind::Other
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
            .recv_using_postcard::<GetCharSomeResponse>()
            .map_err(|_| nb::Error::Other(Error::ReadError(IpcError::GotInvalidResponse)))?;
        resp.val.ok_or(nb::Error::WouldBlock)
    }
}

impl serial::Write<u8> for Client {
    // TODO dont' block?
    fn write(&mut self, val: u8) -> nb::Result<(), Self::Error> {
        let req = Request::PutChar { val };
        self.channel
            .pp_call(MessageInfo::send_using_postcard(req).unwrap())
            .recv_empty()
            .map_err(|_| nb::Error::Other(Error::WriteError(IpcError::GotInvalidResponse)))?;
        Ok(())
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        todo!()
    }
}

// XXX There's already an implementation of fmt::Write for serial::Write
// in embedded_hal::fmt, but I'm not clear on how to use it.
impl fmt::Write for Client {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        s.as_bytes().iter().copied().for_each(|b| {
            let _ = self.write(b);
        });
        Ok(())
    }
}
