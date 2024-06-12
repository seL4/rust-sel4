//
// Copyright 2024, Colias Group, LLC
// Copyright 2023, Galois, Inc.
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_driver_traits::MacAddress;
use sel4_microkit::{Channel, MessageInfo};
use sel4_microkit_message::MessageInfoExt as _;

use super::common::*;

pub struct Client {
    channel: Channel,
}

impl Client {
    pub fn new(channel: Channel) -> Self {
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
