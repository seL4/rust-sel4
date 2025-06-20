//
// Copyright 2023, Colias Group, LLC
// Copyright 2023, Galois, Inc.
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::convert::Infallible;

use embedded_hal_nb::nb;
use embedded_hal_nb::serial;
use heapless::Deque;

use sel4_driver_interfaces::HandleInterrupt;
use sel4_microkit::{Channel, ChannelSet, Handler, MessageInfo};
use sel4_microkit_simple_ipc as simple_ipc;

use super::message_types::*;

/// Handle messages using an implementor of [serial::Read<u8>] and [serial::Write<u8>].
#[derive(Clone, Debug)]
pub struct HandlerImpl<Driver, const READ_BUF_SIZE: usize = 256> {
    /// Driver implementing [serial::Read<u8>] and [serial::Write<u8>].
    driver: Driver,
    /// Channel for this component.
    serial: Channel,
    /// Channel for client component.
    client: Channel,
    /// Read buffer.
    buffer: Deque<u8, READ_BUF_SIZE>,
    /// Whether to notify client.
    notify: bool,
}

impl<Driver, const READ_BUF_SIZE: usize> HandlerImpl<Driver, READ_BUF_SIZE>
where
    Driver: serial::Read<u8> + serial::Write<u8> + HandleInterrupt,
{
    pub fn new(driver: Driver, serial: Channel, client: Channel) -> Self {
        Self {
            driver,
            serial,
            client,
            buffer: Deque::new(),
            notify: true,
        }
    }
}

impl<Driver> Handler for HandlerImpl<Driver>
where
    Driver: serial::Read<u8> + serial::Write<u8> + HandleInterrupt,
{
    type Error = Infallible;

    fn notified(&mut self, channels: ChannelSet) -> Result<(), Self::Error> {
        if channels.contains(self.serial) {
            while !self.buffer.is_full() {
                match self.driver.read() {
                    Ok(v) => {
                        self.buffer.push_back(v).unwrap();
                    }
                    Err(err) => {
                        if let nb::Error::Other(err) = err {
                            // TODO somehow inform the client
                            log::debug!("read error: {err:?}")
                        }
                        break;
                    }
                }
            }
            self.driver.handle_interrupt();
            self.serial.irq_ack().unwrap();
            if self.notify {
                self.client.notify();
                self.notify = false;
            }
        } else {
            panic!("unexpected channels: {}", channels.display());
        }
        Ok(())
    }

    fn protected(
        &mut self,
        channel: Channel,
        msg_info: MessageInfo,
    ) -> Result<MessageInfo, Self::Error> {
        if channel == self.client {
            Ok(match simple_ipc::recv::<Request>(msg_info) {
                Ok(req) => {
                    let resp = match req {
                        Request::Read => {
                            let v = self.buffer.pop_front();
                            if v.is_some() {
                                self.notify = true;
                            }
                            Ok(SuccessResponse::Read(v.into()))
                        }
                        Request::Write(c) => NonBlocking::from_nb_result(self.driver.write(c))
                            .map(SuccessResponse::Write)
                            .map_err(|_| ErrorResponse::WriteError),
                        Request::Flush => NonBlocking::from_nb_result(self.driver.flush())
                            .map(SuccessResponse::Flush)
                            .map_err(|_| ErrorResponse::FlushError),
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
