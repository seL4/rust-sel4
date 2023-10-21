//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_microkit::MessageInfo;
use sel4_microkit_message::MessageInfoExt as _;

use microkit_http_server_example_sp804_driver_interface_types::*;

pub struct TimerClient {
    channel: sel4_microkit::Channel,
}

impl TimerClient {
    pub fn new(channel: sel4_microkit::Channel) -> Self {
        Self { channel }
    }

    pub fn now(&self) -> Microseconds {
        let req = Request::Now;
        let resp: NowResponse = self
            .channel
            .pp_call(MessageInfo::send_using_postcard(req).unwrap())
            .recv_using_postcard()
            .unwrap();
        resp.micros
    }

    pub fn set_timeout(&self, relative_micros: Microseconds) {
        let req = Request::SetTimeout { relative_micros };
        self.channel
            .pp_call(MessageInfo::send_using_postcard(req).unwrap())
            .recv_empty()
            .unwrap();
    }

    #[allow(dead_code)]
    pub fn clear_timeout(&self) {
        let req = Request::ClearTimeout;
        self.channel
            .pp_call(MessageInfo::send_using_postcard(req).unwrap())
            .recv_empty()
            .unwrap();
    }
}
