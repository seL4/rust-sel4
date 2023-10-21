//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_microkit::MessageInfo;
use sel4_microkit_message::MessageInfoExt as _;

use microkit_http_server_example_virtio_net_driver_interface_types::*;

pub struct NetClient {
    channel: sel4_microkit::Channel,
}

impl NetClient {
    pub fn new(channel: sel4_microkit::Channel) -> Self {
        Self { channel }
    }

    pub fn get_mac_address(&self) -> MacAddress {
        let req = Request::GetMacAddress;
        let resp: GetMacAddressResponse = self
            .channel
            .pp_call(MessageInfo::send_using_postcard(req).unwrap())
            .recv_using_postcard()
            .unwrap();
        resp.mac_address
    }
}
