#![no_std]
#![no_main]
#![feature(never_type)]

use heapless::Deque;

use sel4cp::message::{MessageInfo, NoMessageValue, StatusMessageLabel};
use sel4cp::{memory_region_symbol, protection_domain, Channel, Handler};

use banscii_pl011_driver_interface_types::*;

mod device;

use device::{Pl011Device, Pl011RegisterBlock};

const DEVICE: Channel = Channel::new(0);
const ASSISTANT: Channel = Channel::new(1);

#[protection_domain]
fn init() -> ThisHandler {
    let device = unsafe {
        Pl011Device::new(
            memory_region_symbol!(pl011_register_block: *mut Pl011RegisterBlock).as_ptr(),
        )
    };
    device.init();
    ThisHandler {
        device,
        buffer: Deque::new(),
        notify: true,
    }
}

struct ThisHandler {
    device: Pl011Device,
    buffer: Deque<u8, 256>,
    notify: bool,
}

impl Handler for ThisHandler {
    type Error = !;

    fn notified(&mut self, channel: Channel) -> Result<(), Self::Error> {
        match channel {
            DEVICE => {
                while let Some(c) = self.device.get_char() {
                    if self.buffer.push_back(c).is_err() {
                        break;
                    }
                }
                self.device.handle_irq();
                DEVICE.irq_ack().unwrap();
                if self.notify {
                    ASSISTANT.notify();
                    self.notify = false;
                }
            }
            _ => {
                unreachable!()
            }
        }
        Ok(())
    }

    fn protected(
        &mut self,
        channel: Channel,
        msg_info: MessageInfo,
    ) -> Result<MessageInfo, Self::Error> {
        Ok(match channel {
            ASSISTANT => match msg_info.label().try_into().ok() {
                Some(RequestTag::PutChar) => match msg_info.recv() {
                    Ok(PutCharRequest { val }) => {
                        self.device.put_char(val);
                        MessageInfo::send(StatusMessageLabel::Ok, NoMessageValue)
                    }
                    Err(_) => MessageInfo::send(StatusMessageLabel::Error, NoMessageValue),
                },
                Some(RequestTag::GetChar) => match self.buffer.pop_front() {
                    Some(val) => {
                        MessageInfo::send(GetCharResponseTag::Some, GetCharSomeResponse { val })
                    }
                    None => {
                        self.notify = true;
                        MessageInfo::send(GetCharResponseTag::None, NoMessageValue)
                    }
                },
                None => MessageInfo::send(StatusMessageLabel::Error, NoMessageValue),
            },
            _ => {
                unreachable!()
            }
        })
    }
}
