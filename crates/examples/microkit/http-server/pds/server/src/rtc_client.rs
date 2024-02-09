//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_microkit::MessageInfo;
use sel4_microkit_message::MessageInfoExt as _;

use microkit_http_server_example_pl031_driver_interface_types::*;

pub struct RtcClient {
    channel: sel4_microkit::Channel,
}

impl RtcClient {
    pub fn new(channel: sel4_microkit::Channel) -> Self {
        Self { channel }
    }

    pub fn now(&self) -> Seconds {
        let req = Request::Now;
        let resp: NowResponse = self
            .channel
            .pp_call(MessageInfo::send_using_postcard(req).unwrap())
            .recv_using_postcard()
            .unwrap();
        resp.unix_time
    }
}
