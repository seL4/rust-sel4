//
// Copyright 2023, Colias Group, LLC
// Copyright 2023, Galois, Inc.
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::fmt;

use embedded_hal_nb::nb;
use embedded_hal_nb::serial::{self, Write as _};
use heapless::Deque;
use serde::{Deserialize, Serialize};

use sel4_microkit::{Channel, Handler, MessageInfo};
use sel4_microkit_message::MessageInfoExt;

/// Handle messages using an implementor of [serial::Read<u8>] and [serial::Write<u8>].
#[derive(Clone, Debug)]
pub struct SerialHandler<Device, const READ_BUF_SIZE: usize = 256> {
    /// Device implementing [serial::Read<u8>] and [serial::Write<u8>].
    device: Device,
    /// Channel for this component.
    serial: Channel,
    /// Channel for client component.
    client: Channel,
    /// Read buffer.
    buffer: Deque<u8, READ_BUF_SIZE>,
    /// Whether to notify client.
    notify: bool,
}

impl<Device, const READ_BUF_SIZE: usize> SerialHandler<Device, READ_BUF_SIZE>
where
    Device: serial::Read<u8> + serial::Write<u8> + IrqDevice,
{
    pub fn new(device: Device, serial: Channel, client: Channel) -> Self {
        Self {
            device,
            serial,
            client,
            buffer: Deque::new(),
            notify: true,
        }
    }
}

pub trait IrqDevice {
    fn handle_irq(&self);
}

#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum SerialHandlerError<E> {
    DeviceError(E),
    BufferFull,
    // XXX Other errors?
}

impl<E: fmt::Display> fmt::Display for SerialHandlerError<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SerialHandlerError::DeviceError(err) => write!(f, "device error: {err}"),
            SerialHandlerError::BufferFull => write!(f, "buffer full"),
        }
    }
}

impl<Device> Handler for SerialHandler<Device>
where
    Device: serial::Read<u8> + serial::Write<u8> + IrqDevice,
    <Device as serial::ErrorType>::Error: fmt::Display,
{
    type Error = SerialHandlerError<Device::Error>;

    fn notified(&mut self, channel: Channel) -> Result<(), Self::Error> {
        // TODO Handle errors
        if channel == self.serial {
            while let Ok(c) = self.device.read() {
                if let Err(_) = self.buffer.push_back(c) {
                    return Err(SerialHandlerError::BufferFull);
                }
            }
            self.device.handle_irq();
            self.serial.irq_ack().unwrap();
            if self.notify {
                self.client.notify();
                self.notify = false;
            }
        } else {
            panic!("unexpected channel: {channel:?}");
        }
        Ok(())
    }

    fn protected(
        &mut self,
        channel: Channel,
        msg_info: MessageInfo,
    ) -> Result<MessageInfo, Self::Error> {
        // TODO Handle errors
        if channel == self.client {
            Ok(match msg_info.recv_using_postcard::<Request>() {
                Ok(req) => match req {
                    Request::PutChar { val } => {
                        nb::block!(self.device.write(val)).unwrap();
                        MessageInfo::send_empty()
                    }
                    Request::GetChar => {
                        let val = self.buffer.pop_front();
                        if val.is_some() {
                            self.notify = true;
                        }
                        MessageInfo::send_using_postcard(GetCharSomeResponse { val }).unwrap()
                    }
                },
                Err(_) => MessageInfo::send_unspecified_error(),
            })
        } else {
            panic!("unexpected channel: {channel:?}");
        }
    }
}

/// Device-independent embedded_hal::serial interface to a serial-device
/// component. Interact with it using [serial::Read], [serial::Write],
/// and [fmt::Write].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SerialDriver {
    pub channel: Channel,
}

impl SerialDriver {
    pub fn new(channel: Channel) -> Self {
        SerialDriver { channel }
    }
}

#[derive(Clone, Debug)]
pub enum SerialDriverError {
    ReadError(IpcError),
    WriteError(IpcError),
}

#[derive(Clone, Debug)]
pub enum IpcError {
    GotError,
    GotInvalidResponse,
}

impl serial::Error for SerialDriverError {
    fn kind(&self) -> serial::ErrorKind {
        serial::ErrorKind::Other
    }
}

impl serial::ErrorType for SerialDriver {
    type Error = SerialDriverError;
}

impl serial::Read<u8> for SerialDriver {
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        let req = Request::GetChar;
        let resp = self
            .channel
            .pp_call(MessageInfo::send_using_postcard(req).unwrap())
            .recv_using_postcard::<GetCharSomeResponse>()
            .map_err(|_| {
                nb::Error::Other(SerialDriverError::ReadError(IpcError::GotInvalidResponse))
            })?;
        resp.val.ok_or(nb::Error::WouldBlock)
    }
}

impl serial::Write<u8> for SerialDriver {
    // TODO dont' block?
    fn write(&mut self, val: u8) -> nb::Result<(), Self::Error> {
        let req = Request::PutChar { val };
        self.channel
            .pp_call(MessageInfo::send_using_postcard(req).unwrap())
            .recv_empty()
            .map_err(|_| {
                nb::Error::Other(SerialDriverError::WriteError(IpcError::GotInvalidResponse))
            })?;
        Ok(())
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        todo!()
    }
}

// XXX There's already an implementation of fmt::Write for serial::Write
// in embedded_hal::fmt, but I'm not clear on how to use it.
impl fmt::Write for SerialDriver {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        s.as_bytes().iter().copied().for_each(|b| {
            let _ = self.write(b);
        });
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    PutChar { val: u8 },
    GetChar,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetCharSomeResponse {
    pub val: Option<u8>,
}
