#![no_std]
#![no_main]
#![feature(never_type)]

extern crate alloc;

use core::str;

use alloc::vec::Vec;

use sel4cp::*;

use banscii_assistant_core::Draft;

const PL011_DRIVER: Channel = Channel::new(0);

#[main]
fn main() -> ThisHandler {
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
                            create(&self.buffer);
                            self.buffer.clear();
                        }
                    } else {
                        let c: char = b.into();
                        if c.is_ascii() && !c.is_ascii_control() {
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

fn put_char(c: u8) {
    set_mr(0, c as MessageValue);
    let msg_info = PL011_DRIVER.pp_call(MessageInfo::new(0, 1));
    assert_eq!(msg_info.label(), 0);
    assert_eq!(msg_info.count(), 0);
}

fn get_char() -> Option<u8> {
    let msg_info = PL011_DRIVER.pp_call(MessageInfo::new(1, 0));
    match msg_info.label() {
        0 => {
            assert_eq!(msg_info.count(), 0);
            None
        }
        1 => {
            assert_eq!(msg_info.count(), 1);
            Some(get_mr(0) as u8)
        }
        _ => {
            panic!()
        }
    }
}
