#![no_std]
#![no_main]
#![feature(never_type)]

use heapless::Deque;

use sel4cp::{memory_region_symbol, protection_domain, Channel, Handler, MessageInfo};
use sel4cp_message::{MessageInfoExt as _, NoMessageValue, StatusMessageLabel};

use banscii_pl011_driver_core::Driver;
use banscii_pl011_driver_interface_types::*;

const DEVICE: Channel = Channel::new(0);
const ASSISTANT: Channel = Channel::new(1);

#[protection_domain]
fn init() -> HandlerImpl {
    let driver =
        unsafe { Driver::new(memory_region_symbol!(pl011_register_block: *mut ()).as_ptr()) };
    HandlerImpl {
        driver,
        buffer: Deque::new(),
        notify: true,
    }
}

struct HandlerImpl {
    driver: Driver,
    buffer: Deque<u8, 256>,
    notify: bool,
}

impl Handler for HandlerImpl {
    type Error = !;

    fn notified(&mut self, channel: Channel) -> Result<(), Self::Error> {
        match channel {
            DEVICE => {
                while let Some(c) = self.driver.get_char() {
                    if self.buffer.push_back(c).is_err() {
                        break;
                    }
                }
                self.driver.handle_interrupt();
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
                        self.driver.put_char(val);
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
