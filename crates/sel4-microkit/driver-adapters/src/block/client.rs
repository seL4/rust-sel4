//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_driver_interfaces::block::GetBlockDeviceLayout;
use sel4_microkit::Channel;
use sel4_microkit_simple_ipc as simple_ipc;

use super::message_types::*;

pub struct Client {
    channel: Channel,
}

impl Client {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }

    fn request(&self, req: Request) -> Result<SuccessResponse, Error> {
        simple_ipc::call::<_, Response>(self.channel, req)
            .map_err(|_| Error::InvalidResponse)?
            .map_err(Error::ErrorResponse)
    }
}

impl GetBlockDeviceLayout for Client {
    type Error = Error;

    fn get_block_size(&mut self) -> Result<usize, Self::Error> {
        match self.request(Request::GetBlockSize)? {
            SuccessResponse::GetBlockSize(size) => Ok(size),
            _ => Err(Error::UnexpectedResponse),
        }
    }

    fn get_num_blocks(&mut self) -> Result<u64, Self::Error> {
        match self.request(Request::GetNumBlocks)? {
            SuccessResponse::GetNumBlocks(n) => Ok(n),
            _ => Err(Error::UnexpectedResponse),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Error {
    ErrorResponse(ErrorResponse),
    InvalidResponse,
    UnexpectedResponse,
}
