//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]
#![feature(never_type)]

use core::time::Duration;

use sel4_microkit::{memory_region_symbol, protection_domain, var, Channel, Handler, MessageInfo};
use sel4_microkit_message::MessageInfoExt as _;

use microkit_http_server_example_sp804_driver_core::Driver;
use microkit_http_server_example_sp804_driver_interface_types::*;

const DEVICE: Channel = Channel::new(0);
const CLIENT: Channel = Channel::new(1);

#[protection_domain]
fn init() -> HandlerImpl {
    let driver = unsafe {
        Driver::new(
            memory_region_symbol!(sp804_mmio_vaddr: *mut ()).as_ptr(),
            (*var!(freq: usize = 0)).try_into().unwrap(),
        )
    };
    HandlerImpl { driver }
}

struct HandlerImpl {
    driver: Driver,
}

impl Handler for HandlerImpl {
    type Error = !;

    fn notified(&mut self, channel: Channel) -> Result<(), Self::Error> {
        match channel {
            DEVICE => {
                self.driver.handle_interrupt();
                DEVICE.irq_ack().unwrap();
                CLIENT.notify();
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
            CLIENT => match msg_info.recv_using_postcard::<Request>() {
                Ok(req) => match req {
                    Request::Now => {
                        let now = self.driver.now();
                        MessageInfo::send_using_postcard(NowResponse {
                            micros: now.as_micros().try_into().unwrap(),
                        })
                        .unwrap()
                    }
                    Request::SetTimeout { relative_micros } => {
                        self.driver
                            .set_timeout(Duration::from_micros(relative_micros));
                        MessageInfo::send_empty()
                    }
                    Request::ClearTimeout => {
                        self.driver.clear_timeout();
                        MessageInfo::send_empty()
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
