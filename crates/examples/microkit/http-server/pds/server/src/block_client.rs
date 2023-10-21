//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_microkit::MessageInfo;
use sel4_microkit_message::MessageInfoExt as _;

use microkit_http_server_example_virtio_blk_driver_interface_types::*;

pub struct BlockClient {
    channel: sel4_microkit::Channel,
}

impl BlockClient {
    pub fn new(channel: sel4_microkit::Channel) -> Self {
        Self { channel }
    }

    pub fn get_num_blocks(&self) -> u64 {
        let req = Request::GetNumBlocks;
        let resp: GetNumBlocksResponse = self
            .channel
            .pp_call(MessageInfo::send_using_postcard(req).unwrap())
            .recv_using_postcard()
            .unwrap();
        resp.num_blocks
    }
}
