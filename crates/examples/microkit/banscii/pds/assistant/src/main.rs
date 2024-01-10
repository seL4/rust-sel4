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
use core::fmt;
use core::fmt::Write;
use core::mem;
use core::str;

use sel4_externally_shared::{
    access::{ReadOnly, ReadWrite},
    ExternallySharedRef, ExternallySharedRefExt,
};
use sel4_microkit::{
    memory_region_symbol, protection_domain, Channel, Handler, Infallible, MessageInfo,
};
use sel4_microkit_message::MessageInfoExt as _;

use banscii_artist_interface_types as artist;
use banscii_assistant_core::Draft;
use banscii_pl011_driver_interface_types as pl011_driver;

const PL011_DRIVER: Channel = Channel::new(0);
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

    prompt();

    HandlerImpl {
        region_in,
        region_out,
        buffer: Vec::new(),
    }
}

struct HandlerImpl {
    region_in: ExternallySharedRef<'static, [u8], ReadOnly>,
    region_out: ExternallySharedRef<'static, [u8], ReadWrite>,
    buffer: Vec<u8>,
}

impl Handler for HandlerImpl {
    type Error = Infallible;

    fn notified(&mut self, channel: Channel) -> Result<(), Self::Error> {
        match channel {
            PL011_DRIVER => {
                while let Some(b) = get_char() {
                    if let b'\n' | b'\r' = b {
                        newline();
                        if !self.buffer.is_empty() {
                            self.try_create();
                        }
                        prompt();
                    } else {
                        let c = char::from(b);
                        if c.is_ascii() && !c.is_ascii_control() {
                            if self.buffer.len() == MAX_SUBJECT_LEN {
                                writeln!(PutCharWrite, "\n(char limit reached)").unwrap();
                                self.try_create();
                                prompt();
                            }
                            put_char(b);
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
                writeln!(PutCharWrite, "error: input is not valid utf-8").unwrap();
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

        newline();

        for row in 0..height {
            for col in 0..width {
                let i = row * width + col;
                let b = pixel_data[i];
                put_char(b);
            }
            newline();
        }

        newline();

        writeln!(PutCharWrite, "Signature:").unwrap();
        for line in signature.chunks(32) {
            writeln!(PutCharWrite, "{}", hex::encode(line)).unwrap();
        }

        newline();
    }
}

fn prompt() {
    write!(PutCharWrite, "banscii> ").unwrap();
}

fn newline() {
    writeln!(PutCharWrite).unwrap();
}

fn get_char() -> Option<u8> {
    let req = pl011_driver::Request::GetChar;
    let resp: pl011_driver::GetCharSomeResponse = PL011_DRIVER
        .pp_call(MessageInfo::send_using_postcard(req).unwrap())
        .recv_using_postcard()
        .unwrap();
    resp.val
}

fn put_char(val: u8) {
    let req = pl011_driver::Request::PutChar { val };
    PL011_DRIVER
        .pp_call(MessageInfo::send_using_postcard(req).unwrap())
        .recv_empty()
        .unwrap();
}

fn put_str(s: &str) {
    s.as_bytes().iter().copied().for_each(put_char)
}

struct PutCharWrite;

impl fmt::Write for PutCharWrite {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        put_str(s);
        Ok(())
    }
}
