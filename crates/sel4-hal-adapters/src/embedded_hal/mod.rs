//
// Copyright 2023, Colias Group, LLC
// Copyright 2023, Galois, Inc.
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use num_enum::{IntoPrimitive, TryFromPrimitive};
use zerocopy::{AsBytes, FromBytes};
use core::fmt;
use heapless::Deque;

use embedded_hal::serial;
use embedded_hal::prelude::_embedded_hal_serial_Write;
use sel4cp::{Channel, Handler};
use sel4cp::message::{MessageInfo, NoMessageValue, StatusMessageLabel};
use sel4cp::message::MessageInfoRecvError;

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

impl<Device> SerialHandler<Device>
where
    Device: serial::Read<u8> + serial::Write<u8> + IrqDevice
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
pub enum SerialHandlerError<Device>
where
    Device: serial::Read<u8> + serial::Write<u8>,
    <Device as serial::Read<u8>>::Error: core::fmt::Debug + Clone,
    <Device as serial::Write<u8>>::Error: core::fmt::Debug + Clone,
{
    ReadError(<Device as serial::Read<u8>>::Error),
    WriteError(<Device as serial::Write<u8>>::Error),
    BufferFull,
    // XXX Other errors?
}

impl<Device> fmt::Display for SerialHandlerError<Device>
where
    Device: serial::Read<u8> + serial::Write<u8>,
    <Device as serial::Read<u8>>::Error: core::fmt::Debug + Clone,
    <Device as serial::Write<u8>>::Error: core::fmt::Debug + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SerialHandlerError::ReadError(_) => write!(f, "SerialHandlerError::ReadError"),
            SerialHandlerError::WriteError(_) => write!(f, "SerialHandlerError::WriteError"),
            SerialHandlerError::BufferFull => write!(f, "SerialHandlerError::BufferFull"),
        }
    }
}

impl<Device> Handler for SerialHandler<Device>
where
    Device: serial::Read<u8> + serial::Write<u8> + IrqDevice,
    <Device as serial::Read<u8>>::Error: core::fmt::Debug + Clone,
    <Device as serial::Write<u8>>::Error: core::fmt::Debug + Clone,
{
    type Error = SerialHandlerError<Device>;

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
            unreachable!() // XXX Is this actually unreachable?
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
            match msg_info.label().try_into().ok() /* XXX Handle errors? */ {
                Some(RequestTag::Write) => match msg_info.recv() {
                    Ok(WriteRequest { val }) => {
                        // Blocking write
                        while let Err(nb::Error::WouldBlock) = self.device.write(val) {}
                        Ok(MessageInfo::send(StatusMessageLabel::Ok, NoMessageValue))
                    }
                    Err(_) => Ok(MessageInfo::send(StatusMessageLabel::Error, NoMessageValue)),
                },
                Some(RequestTag::Read) => match self.buffer.pop_front() {
                    Some(val) => {
                        Ok(MessageInfo::send(ReadResponseTag::Some, ReadSomeResponse { val }))
                    }
                    None => {
                        self.notify = true;
                        Ok(MessageInfo::send(ReadResponseTag::None, NoMessageValue))
                    }
                },
                None => Ok(MessageInfo::send(StatusMessageLabel::Error, NoMessageValue)),
            }
        } else {
            unreachable!() // XXX Is this actually unreachable?
        }
    }
}

/// Device-independent embedded_hal::serial interface to a serial-device
/// component. Interact with it using [serial::Read], [serial::Write],
/// and [fmt::Write].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SerialDriver {
    pub channel: Channel
}

impl SerialDriver {
    pub fn new(channel: Channel) -> Self {
        SerialDriver { channel }
    }
}

#[derive(Clone, Debug)]
pub enum ReadError {
    RecvError(MessageInfoRecvError),
    InvalidResponse,
    EOF,
}

impl serial::Read<u8> for SerialDriver {
    type Error = ReadError;

    // XXX Unclear if this blocks or how to prevent it from doing so...
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        let msg_info = self.channel
            .pp_call(MessageInfo::send(RequestTag::Read, NoMessageValue));

        match msg_info.label().try_into() {
            Ok(ReadResponseTag::Some) => match msg_info.recv() {
                Ok(ReadSomeResponse { val }) => Ok(val),
                Err(e) => Err(nb::Error::Other(ReadError::RecvError(e))),
            },
            Ok(ReadResponseTag::None) => Err(nb::Error::Other(ReadError::EOF)),
            Err(_) => Err(nb::Error::Other(ReadError::InvalidResponse)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WriteError {
    SendError,
    InvalidResponse,
}

impl serial::Write<u8> for SerialDriver {
    type Error = WriteError;

    // XXX Unclear if this blocks or how to prevent it from doing so...
    fn write(&mut self, val: u8) -> nb::Result<(), Self::Error> {
        let msg_info = self.channel
            .pp_call(MessageInfo::send(RequestTag::Write, WriteRequest { val }));

        match msg_info.label().try_into() {
            Ok(StatusMessageLabel::Ok) => Ok(()),
            Ok(StatusMessageLabel::Error) => Err(nb::Error::Other(WriteError::SendError)),
            Err(_) => Err(nb::Error::Other(WriteError::InvalidResponse)),
        }
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        todo!()
    }
}

// XXX There's already an implementation of core::fmt::Write for serial::Write
// in embedded_hal::fmt, but I'm not clear on how to use it.
impl fmt::Write for SerialDriver {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        s.as_bytes().iter().copied().for_each(|b| { let _ = self.write(b); });
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[cfg_attr(target_pointer_width = "32", repr(u32))]
#[cfg_attr(target_pointer_width = "64", repr(u64))]
pub enum RequestTag {
    Write,
    Read,
}

#[derive(Clone, Copy, PartialEq, Eq, AsBytes, FromBytes)]
#[repr(C)]
pub struct WriteRequest {
    pub val: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[cfg_attr(target_pointer_width = "32", repr(u32))]
#[cfg_attr(target_pointer_width = "64", repr(u64))]
pub enum ReadResponseTag {
    None,
    Some,
}

#[derive(Clone, Copy, PartialEq, Eq, AsBytes, FromBytes)]
#[repr(C)]
pub struct ReadSomeResponse {
    pub val: u8,
}
