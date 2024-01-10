//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use heapless::Deque;

use sel4_microkit::{
    memory_region_symbol, protection_domain, Channel, Handler, Infallible, MessageInfo,
};
use sel4_microkit_message::MessageInfoExt as _;

use banscii_pl011_driver_core::Driver;
use banscii_pl011_driver_interface_types::*;

const DEVICE: Channel = Channel::new(0);
const ASSISTANT: Channel = Channel::new(1);

#[protection_domain]
fn init() -> HandlerImpl {
    let driver =
        unsafe { Driver::new(memory_region_symbol!(pl011_register_block: *mut ()).as_ptr()) };
    HandlerImpl {
        driver,
        buffer: Deque::new(),
        notify: true,
    }
}

struct HandlerImpl {
    driver: Driver,
    buffer: Deque<u8, 256>,
    notify: bool,
}

impl Handler for HandlerImpl {
    type Error = Infallible;

    fn notified(&mut self, channel: Channel) -> Result<(), Self::Error> {
        match channel {
            DEVICE => {
                while let Some(c) = self.driver.get_char() {
                    if self.buffer.push_back(c).is_err() {
                        break;
                    }
                }
                self.driver.handle_interrupt();
                DEVICE.irq_ack().unwrap();
                if self.notify {
                    ASSISTANT.notify();
                    self.notify = false;
                }
            }
            _ => {
                unreachable!()
            }
        }
        Ok(())
    }

    fn protected(
        &mut self,
        channel: Channel,
        msg_info: MessageInfo,
    ) -> Result<MessageInfo, Self::Error> {
        Ok(match channel {
            ASSISTANT => match msg_info.recv_using_postcard::<Request>() {
                Ok(req) => match req {
                    Request::PutChar { val } => {
                        self.driver.put_char(val);
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
            },
            _ => {
                unreachable!()
            }
        })
    }
}
