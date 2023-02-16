#![no_std]
#![no_main]
#![feature(never_type)]

extern crate alloc;

use core::str;

use alloc::vec::Vec;

use sel4cp::*;

use banscii_assistant_core::Draft;
use banscii_pl011_driver_interface_types as driver;

const PL011_DRIVER: Channel = Channel::new(0);

const MAX_SUBJECT_LEN: usize = 16;

#[main]
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
    let palette = b"@%#x+=:-. ";
    for row in 0..draft.height {
        for col in 0..draft.width {
            let i = row * draft.width + col;
            let v = draft.pixel_data[i];
            let c = palette[usize::from(v / 26)];
            put_char(c);
        }
        put_char(b'\n');
    }
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
