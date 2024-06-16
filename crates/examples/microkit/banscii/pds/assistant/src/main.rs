//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use core::fmt::Write;
use core::mem;
use core::str;

use embedded_hal_nb::serial::{self, Read as _, Write as _};

use sel4_externally_shared::{
    access::{ReadOnly, ReadWrite},
    ExternallySharedRef, ExternallySharedRefExt,
};
use sel4_microkit::{
    memory_region_symbol, protection_domain, Channel, Handler, Infallible, MessageInfo,
};
use sel4_microkit_driver_adapters::serial::client::{
    Client as SerialClient, Error as SerialClientError,
};
use sel4_microkit_message::MessageInfoExt as _;

use banscii_artist_interface_types as artist;
use banscii_assistant_core::Draft;

const SERIAL_DRIVER: Channel = Channel::new(0);
const ARTIST: Channel = Channel::new(1);

const REGION_SIZE: usize = 0x4_000;

const MAX_SUBJECT_LEN: usize = 16;

#[protection_domain(heap_size = 0x10000)]
fn init() -> impl Handler {
    let region_in = unsafe {
        ExternallySharedRef::new(memory_region_symbol!(region_in_start: *mut [u8], n = REGION_SIZE))
    };

    let region_out = unsafe {
        ExternallySharedRef::new(
            memory_region_symbol!(region_out_start: *mut [u8], n = REGION_SIZE),
        )
    };

    let mut this = HandlerImpl {
        serial_client: SerialClient::new(SERIAL_DRIVER),
        region_in,
        region_out,
        buffer: Vec::new(),
    };

    this.prompt();

    this
}

struct HandlerImpl {
    serial_client: SerialClient,
    region_in: ExternallySharedRef<'static, [u8], ReadOnly>,
    region_out: ExternallySharedRef<'static, [u8], ReadWrite>,
    buffer: Vec<u8>,
}

impl Handler for HandlerImpl {
    type Error = Infallible;

    fn notified(&mut self, channel: Channel) -> Result<(), Self::Error> {
        match channel {
            SERIAL_DRIVER => {
                while let Ok(b) = self.serial_client.read() {
                    if let b'\n' | b'\r' = b {
                        self.newline();
                        if !self.buffer.is_empty() {
                            self.try_create();
                        }
                        self.prompt();
                    } else {
                        let c = char::from(b);
                        if c.is_ascii() && !c.is_ascii_control() {
                            if self.buffer.len() == MAX_SUBJECT_LEN {
                                writeln!(self.writer(), "\n(char limit reached)").unwrap();
                                self.try_create();
                                self.prompt();
                            }
                            self.serial_client.write(b).unwrap();
                            self.buffer.push(b);
                        }
                    }
                }
            }
            _ => {
                unreachable!()
            }
        }
        Ok(())
    }
}

impl HandlerImpl {
    fn try_create(&mut self) {
        let mut buffer = Vec::new();
        mem::swap(&mut buffer, &mut self.buffer);
        match str::from_utf8(&buffer) {
            Ok(subject) => {
                self.create(subject);
            }
            Err(_) => {
                writeln!(self.writer(), "error: input is not valid utf-8").unwrap();
            }
        };
        self.buffer.clear();
    }

    fn create(&mut self, subject: &str) {
        let draft = Draft::new(subject);

        let draft_start = 0;
        let draft_size = draft.pixel_data.len();
        let draft_end = draft_start + draft_size;

        self.region_out
            .as_mut_ptr()
            .index(draft_start..draft_end)
            .copy_from_slice(&draft.pixel_data);

        let req = artist::Request {
            height: draft.height,
            width: draft.width,
            draft_start,
            draft_size,
        };

        let resp: artist::Response = ARTIST
            .pp_call(MessageInfo::send_using_postcard(req).unwrap())
            .recv_using_postcard()
            .unwrap();

        let height = resp.height;
        let width = resp.width;

        let pixel_data = {
            let mut buf = vec![0; resp.masterpiece_size];
            self.region_in
                .as_ptr()
                .index(resp.masterpiece_start..resp.masterpiece_start + resp.masterpiece_size)
                .copy_into_slice(&mut buf);
            buf
        };

        let signature = {
            let mut buf = vec![0; resp.signature_size];
            self.region_in
                .as_ptr()
                .index(resp.signature_start..resp.signature_start + resp.signature_size)
                .copy_into_slice(&mut buf);
            buf
        };

        self.newline();

        for row in 0..height {
            for col in 0..width {
                let i = row * width + col;
                let b = pixel_data[i];
                self.serial_client.write(b).unwrap();
            }
            self.newline();
        }

        self.newline();

        writeln!(self.writer(), "Signature:").unwrap();
        for line in signature.chunks(32) {
            writeln!(self.writer(), "{}", hex::encode(line)).unwrap();
        }

        self.newline();
    }

    fn prompt(&mut self) {
        write!(self.writer(), "banscii> ").unwrap();
    }

    fn newline(&mut self) {
        writeln!(self.writer()).unwrap();
    }

    fn writer(&mut self) -> &mut dyn serial::Write<Error = SerialClientError> {
        &mut self.serial_client as &mut dyn serial::Write<Error = SerialClientError>
    }
}
