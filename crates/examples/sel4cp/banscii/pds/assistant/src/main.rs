#![no_std]
#![no_main]
#![feature(const_trait_impl)]
#![feature(never_type)]

extern crate alloc;

use alloc::vec::Vec;
use core::fmt;
use core::fmt::Write;
use core::mem;
use core::str;

use sel4cp::memory_region::{
    declare_memory_region, MemoryRegion, ReadOnly, ReadWrite, SharedSliceExt,
};
use sel4cp::message::{MessageInfo, NoMessageLabel, NoMessageValue, StatusMessageLabel};
use sel4cp::{protection_domain, Channel, Handler};

use banscii_artist_interface_types as artist;
use banscii_assistant_core::Draft;
use banscii_pl011_driver_interface_types as driver;

const PL011_DRIVER: Channel = Channel::new(0);
const TALENT: Channel = Channel::new(1);

const REGION_SIZE: usize = 0x4_000;

const MAX_SUBJECT_LEN: usize = 16;

#[protection_domain(heap_size = 0x10000)]
fn init() -> ThisHandler {
    let region_in = unsafe {
        declare_memory_region! {
            <[u8], ReadOnly>(region_in_start, REGION_SIZE)
        }
    };
    let region_out = unsafe {
        declare_memory_region! {
            <[u8], ReadWrite>(region_out_start, REGION_SIZE)
        }
    };

    prompt();

    ThisHandler {
        region_in,
        region_out,
        buffer: Vec::new(),
    }
}

struct ThisHandler {
    region_in: MemoryRegion<[u8], ReadOnly>,
    region_out: MemoryRegion<[u8], ReadWrite>,
    buffer: Vec<u8>,
}

impl Handler for ThisHandler {
    type Error = !;

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

impl ThisHandler {
    fn try_create(&mut self) {
        let mut buffer = Vec::new();
        mem::swap(&mut buffer, &mut self.buffer);
        match str::from_utf8(&buffer) {
            Ok(subject) => {
                self.create(&subject);
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
            .index_mut(draft_start..draft_end)
            .copy_from_slice(&draft.pixel_data);

        let msg_info = TALENT.pp_call(MessageInfo::send(
            NoMessageLabel,
            artist::Request {
                height: draft.height,
                width: draft.width,
                draft_start,
                draft_size,
            },
        ));

        assert_eq!(msg_info.label().try_into(), Ok(StatusMessageLabel::Ok));

        let msg = msg_info.recv::<artist::Response>().unwrap();

        let height = msg.height;
        let width = msg.width;

        let pixel_data = self
            .region_in
            .index(msg.masterpiece_start..msg.masterpiece_start + msg.masterpiece_size)
            .copy_to_vec();

        let signature = self
            .region_in
            .index(msg.signature_start..msg.signature_start + msg.signature_size)
            .copy_to_vec();

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
    writeln!(PutCharWrite, "").unwrap();
}

fn get_char() -> Option<u8> {
    let msg_info = PL011_DRIVER.pp_call(MessageInfo::send(
        driver::RequestTag::GetChar,
        NoMessageValue,
    ));
    match msg_info.label().try_into().ok() {
        Some(driver::GetCharResponseTag::Some) => match msg_info.recv() {
            Ok(driver::GetCharSomeResponse { val }) => Some(val),
            Err(_) => {
                panic!()
            }
        },
        Some(driver::GetCharResponseTag::None) => None,
        _ => {
            panic!()
        }
    }
}

fn put_char(val: u8) {
    let msg_info = PL011_DRIVER.pp_call(MessageInfo::send(
        driver::RequestTag::PutChar,
        driver::PutCharRequest { val },
    ));
    assert_eq!(msg_info.label().try_into(), Ok(StatusMessageLabel::Ok));
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
