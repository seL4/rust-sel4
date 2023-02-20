#![no_std]
#![no_main]
#![feature(never_type)]

extern crate alloc;

use alloc::vec;
use core::ptr;
use core::slice;

use volatile::{access::ReadOnly, Volatile};

use sel4cp::*;

use banscii_talent_interface_types::*;

mod cryptographic_secrets;

const ASSISTANT: Channel = Channel::new(0);

const REGION_SIZE: usize = 0x4_000;

#[used]
#[no_mangle]
#[link_section = ".data"]
static mut region_in_start: *mut u8 = ptr::null_mut();

#[used]
#[no_mangle]
#[link_section = ".data"]
static mut region_out_start: *mut u8 = ptr::null_mut();

fn get_region_in() -> Volatile<&'static mut [u8], ReadOnly> {
    Volatile::new_read_only(unsafe { slice::from_raw_parts_mut(region_in_start, REGION_SIZE) })
}

fn get_region_out() -> Volatile<&'static mut [u8]> {
    Volatile::new(unsafe { slice::from_raw_parts_mut(region_out_start, REGION_SIZE) })
}

#[main(heap_size = 0x10000)]
fn main() -> ThisHandler {
    ThisHandler {}
}

struct ThisHandler {}

impl Handler for ThisHandler {
    type Error = !;

    fn protected(
        &mut self,
        channel: Channel,
        msg_info: MessageInfo,
    ) -> Result<MessageInfo, Self::Error> {
        Ok(match channel {
            ASSISTANT => match msg_info.recv::<Request>() {
                Ok(msg) => {
                    let height = msg.height;
                    let width = msg.width;

                    let draft = {
                        let mut this = vec![0; msg.draft_size];
                        get_region_in()
                            .index(msg.draft_start..msg.draft_start + msg.draft_size)
                            .copy_into_slice(&mut this);
                        this
                    };

                    let masterpiece = {
                        let mut this = draft.clone();
                        let palette = b"@%#x+=:-. ";
                        for row in 0..height {
                            for col in 0..width {
                                let i = row * width + col;
                                let v = draft[i];
                                let c = palette[usize::from(v / 26)];
                                this[i] = c;
                            }
                        }
                        this
                    };

                    let masterpiece_start = 0;
                    let masterpiece_size = masterpiece.len();
                    let masterpiece_end = masterpiece_start + masterpiece_size;
                    get_region_out()
                        .index_mut(masterpiece_start..masterpiece_end)
                        .copy_from_slice(&masterpiece);

                    let signature = cryptographic_secrets::sign(&masterpiece);
                    let signature = signature.as_ref();

                    let signature_start = masterpiece_end;
                    let signature_size = signature.len();
                    let signature_end = signature_start + signature_size;
                    get_region_out()
                        .index_mut(signature_start..signature_end)
                        .copy_from_slice(&signature);

                    MessageInfo::send(
                        StatusMessageLabel::Ok,
                        Response {
                            height,
                            width,
                            masterpiece_start,
                            masterpiece_size,
                            signature_start,
                            signature_size,
                        },
                    )
                }
                Err(_) => MessageInfo::send(StatusMessageLabel::Error, NoMessageValue),
            },
            _ => {
                unreachable!()
            }
        })
    }
}
