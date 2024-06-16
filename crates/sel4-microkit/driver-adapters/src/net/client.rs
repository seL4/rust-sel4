//
// Copyright 2024, Colias Group, LLC
// Copyright 2023, Galois, Inc.
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_driver_interfaces::net::MacAddress;
use sel4_microkit::{Channel, MessageInfo};
use sel4_microkit_message::MessageInfoExt as _;

use super::message_types::*;

pub struct Client {
    channel: Channel,
}

impl Client {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }
    fn request(&self, req: Request) -> Result<SuccessResponse, Error> {
        self.channel
            .pp_call(MessageInfo::send_using_postcard(req).unwrap())
            .recv_using_postcard::<Response>()
            .map_err(|_| Error::InvalidResponse)?
            .map_err(Error::ErrorResponse)
    }

    pub fn get_mac_address(&self) -> Result<MacAddress, Error> {
        match self.request(Request::GetMacAddress)? {
            SuccessResponse::GetMacAddress { mac_address } => Ok(mac_address),
            // _ => Err(Error::UnexpectedResponse),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Error {
    ErrorResponse(ErrorResponse),
    InvalidResponse,
    UnexpectedResponse,
}
