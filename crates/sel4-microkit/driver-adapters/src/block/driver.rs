//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_driver_interfaces::block::GetBlockDeviceLayout;
use sel4_microkit::MessageInfo;
use sel4_microkit_message::MessageInfoExt as _;

use super::message_types::*;

pub fn handle_client_request<T: GetBlockDeviceLayout>(
    dev: &mut T,
    msg_info: MessageInfo,
) -> MessageInfo {
    match msg_info.recv_using_postcard::<Request>() {
        Ok(req) => {
            let resp: Response = match req {
                Request::GetNumBlocks => dev
                    .get_num_blocks()
                    .map(SuccessResponse::GetNumBlocks)
                    .map_err(|_| ErrorResponse::Unspecified),
                Request::GetBlockSize => dev
                    .get_block_size()
                    .map(SuccessResponse::GetBlockSize)
                    .map_err(|_| ErrorResponse::Unspecified),
            };
            MessageInfo::send_using_postcard(resp).unwrap()
        }
        Err(_) => MessageInfo::send_unspecified_error(),
    }
}
