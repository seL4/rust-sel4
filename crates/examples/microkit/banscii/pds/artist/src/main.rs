//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec;

use sel4_externally_shared::{
    access::{ReadOnly, ReadWrite},
    ExternallySharedRef, ExternallySharedRefExt,
};
use sel4_microkit::{
    memory_region_symbol, protection_domain, Channel, Handler, Infallible, MessageInfo,
};
use sel4_microkit_message::MessageInfoExt as _;

use banscii_artist_interface_types::*;

mod artistic_secrets;
mod cryptographic_secrets;

use artistic_secrets::Masterpiece;

const ASSISTANT: Channel = Channel::new(0);

const REGION_SIZE: usize = 0x4_000;

#[protection_domain(heap_size = 0x10000)]
fn init() -> HandlerImpl {
    let region_in = unsafe {
        ExternallySharedRef::new(memory_region_symbol!(region_in_start: *mut [u8], n = REGION_SIZE))
    };

    let region_out = unsafe {
        ExternallySharedRef::new(
            memory_region_symbol!(region_out_start: *mut [u8], n = REGION_SIZE),
        )
    };

    HandlerImpl {
        region_in,
        region_out,
    }
}

struct HandlerImpl {
    region_in: ExternallySharedRef<'static, [u8], ReadOnly>,
    region_out: ExternallySharedRef<'static, [u8], ReadWrite>,
}

impl Handler for HandlerImpl {
    type Error = Infallible;

    fn protected(
        &mut self,
        channel: Channel,
        msg_info: MessageInfo,
    ) -> Result<MessageInfo, Self::Error> {
        Ok(match channel {
            ASSISTANT => match msg_info.recv_using_postcard::<Request>() {
                Ok(req) => {
                    let draft_height = req.height;
                    let draft_width = req.width;
                    let draft = {
                        let mut buf = vec![0; req.draft_size];
                        self.region_in
                            .as_ptr()
                            .index(req.draft_start..req.draft_start + req.draft_size)
                            .copy_into_slice(&mut buf);
                        buf
                    };

                    let masterpiece = Masterpiece::complete(draft_height, draft_width, &draft);

                    let masterpiece_start = 0;
                    let masterpiece_size = masterpiece.pixel_data.len();
                    let masterpiece_end = masterpiece_start + masterpiece_size;

                    self.region_out
                        .as_mut_ptr()
                        .index(masterpiece_start..masterpiece_end)
                        .copy_from_slice(&masterpiece.pixel_data);

                    let signature = cryptographic_secrets::sign(&masterpiece.pixel_data);
                    let signature = signature.as_ref();

                    let signature_start = masterpiece_end;
                    let signature_size = signature.len();
                    let signature_end = signature_start + signature_size;

                    self.region_out
                        .as_mut_ptr()
                        .index(signature_start..signature_end)
                        .copy_from_slice(signature);

                    MessageInfo::send_using_postcard(Response {
                        height: masterpiece.height,
                        width: masterpiece.width,
                        masterpiece_start,
                        masterpiece_size,
                        signature_start,
                        signature_size,
                    })
                    .unwrap()
                }
                Err(_) => MessageInfo::send_unspecified_error(),
            },
            _ => {
                unreachable!()
            }
        })
    }
}
