//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_driver_interfaces::block::GetBlockDeviceLayout;
use sel4_microkit::MessageInfo;
use sel4_microkit_simple_ipc as simple_ipc;

use super::message_types::*;

pub fn handle_client_request<T: GetBlockDeviceLayout>(
    dev: &mut T,
    msg_info: MessageInfo,
) -> MessageInfo {
    match simple_ipc::recv::<Request>(msg_info) {
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
            simple_ipc::send(resp)
        }
        Err(_) => simple_ipc::send_unspecified_error(),
    }
}
