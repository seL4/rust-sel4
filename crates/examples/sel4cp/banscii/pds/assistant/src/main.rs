#![no_std]
#![no_main]
#![feature(const_trait_impl)]
#![feature(never_type)]

extern crate alloc;

use alloc::vec::Vec;
use core::str;

use sel4cp::{memory_region::*, *};

use banscii_assistant_core::Draft;
use banscii_pl011_driver_interface_types as driver;
use banscii_talent_interface_types as talent;

const PL011_DRIVER: Channel = Channel::new(0);
const TALENT: Channel = Channel::new(1);

const MAX_SUBJECT_LEN: usize = 16;

const REGION_SIZE: usize = 0x4_000;

fn get_region_in() -> MemoryRegion<[u8], ReadOnly> {
    unsafe {
        declare_memory_region! {
            <[u8], ReadOnly>(region_in_start, REGION_SIZE)
        }
    }
}

fn get_region_out() -> MemoryRegion<[u8], ReadWrite> {
    unsafe {
        declare_memory_region! {
            <[u8], ReadWrite>(region_out_start, REGION_SIZE)
        }
    }
}

#[main(heap_size = 0x10000)]
fn main() -> ThisHandler {
    prompt();
    ThisHandler { buffer: Vec::new() }
}

struct ThisHandler {
    buffer: Vec<u8>,
}

impl Handler for ThisHandler {
    type Error = !;

    fn notified(&mut self, channel: Channel) -> Result<(), Self::Error> {
        match channel {
            PL011_DRIVER => {
                while let Some(b) = get_char() {
                    if let b'\n' | b'\r' = b {
                        put_char(b'\n');
                        if !self.buffer.is_empty() {
                            self.create();
                        }
                        prompt();
                    } else {
                        let c = char::from(b);
                        if c.is_ascii() && !c.is_ascii_control() {
                            if self.buffer.len() == MAX_SUBJECT_LEN {
                                put_chars(b"\n(char limit reached)\n");
                                self.create();
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
    fn create(&mut self) {
        create(&self.buffer);
        self.buffer.clear();
    }
}

fn create(input: &[u8]) {
    let subject = str::from_utf8(input).unwrap();
    let draft = Draft::new(subject);

    let draft_start = 0;
    let draft_size = draft.pixel_data.len();

    get_region_out()
        .index_mut(draft_start..draft_size)
        .copy_from_slice(&draft.pixel_data);

    let msg_info = TALENT.pp_call(MessageInfo::send(
        NoMessageLabel,
        talent::Request {
            height: draft.height,
            width: draft.width,
            draft_start,
            draft_size,
        },
    ));

    assert_eq!(msg_info.label().try_into(), Ok(StatusMessageLabel::Ok));

    let msg = msg_info.recv::<talent::Response>().unwrap();

    let height = msg.height;
    let width = msg.width;
    let pixel_data = get_region_in()
        .index(msg.masterpiece_start..msg.masterpiece_start + msg.masterpiece_size)
        .copy_to_vec();

    let signature = get_region_in()
        .index(msg.signature_start..msg.signature_start + msg.signature_size)
        .copy_to_vec();

    for row in 0..height {
        for col in 0..width {
            let i = row * width + col;
            let b = pixel_data[i];
            put_char(b);
        }
        put_char(b'\n');
    }

    put_chars(b"Signature: ");
    put_chars(hex::encode(&signature).as_bytes());
    put_char(b'\n');
}

fn prompt() {
    put_chars(b"banscii> ")
}

fn put_chars(vals: &[u8]) {
    for val in vals {
        put_char(*val);
    }
}

fn put_char(val: u8) {
    let msg_info = PL011_DRIVER.pp_call(MessageInfo::send(
        driver::RequestTag::PutChar,
        driver::PutCharRequest { val },
    ));
    assert_eq!(msg_info.label().try_into(), Ok(StatusMessageLabel::Ok));
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
