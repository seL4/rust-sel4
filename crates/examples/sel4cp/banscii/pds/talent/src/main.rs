#![no_std]
#![no_main]
#![feature(const_trait_impl)]
#![feature(never_type)]

extern crate alloc;

use sel4cp::{memory_region::*, *};

use banscii_talent_interface_types::*;

mod cryptographic_secrets;

const ASSISTANT: Channel = Channel::new(0);

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

                    let draft = get_region_in()
                        .index(msg.draft_start..msg.draft_start + msg.draft_size)
                        .copy_to_vec();

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
