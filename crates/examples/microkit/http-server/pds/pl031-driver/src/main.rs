//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use sel4_microkit::{
    memory_region_symbol, protection_domain, Channel, Handler, Infallible, MessageInfo,
};
use sel4_microkit_message::MessageInfoExt as _;

use microkit_http_server_example_pl031_driver_core::Driver;
use microkit_http_server_example_pl031_driver_interface_types::*;

mod config;

use config::channels;

#[protection_domain]
fn init() -> HandlerImpl {
    let driver = unsafe { Driver::new(memory_region_symbol!(pl031_mmio_vaddr: *mut ()).as_ptr()) };
    HandlerImpl { driver }
}

struct HandlerImpl {
    driver: Driver,
}

impl Handler for HandlerImpl {
    type Error = Infallible;

    fn protected(
        &mut self,
        channel: Channel,
        msg_info: MessageInfo,
    ) -> Result<MessageInfo, Self::Error> {
        Ok(match channel {
            channels::CLIENT => match msg_info.recv_using_postcard::<Request>() {
                Ok(req) => match req {
                    Request::Now => {
                        let now = self.driver.now();
                        MessageInfo::send_using_postcard(NowResponse {
                            unix_time: now.as_secs().try_into().unwrap(),
                        })
                        .unwrap()
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
