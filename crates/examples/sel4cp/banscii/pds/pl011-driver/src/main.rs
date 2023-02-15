#![no_std]
#![no_main]
#![feature(never_type)]

use heapless::Deque;

use sel4cp::*;

mod device;

use device::Pl011Device;

#[used]
#[no_mangle]
#[link_section = ".data"]
static mut pl011_mmio_base: usize = 0;

const DEVICE: Channel = Channel::new(0);
const ASSISTANT: Channel = Channel::new(1);

#[main]
fn main() -> ThisHandler {
    let device = Pl011Device::new(unsafe { pl011_mmio_base });
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
                DEVICE.irq_ack().unwrap();
                self.device.handle_interrupt();
                if self.notify {
                    ASSISTANT.notify();
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
            ASSISTANT => match msg_info.label() {
                0 => {
                    assert_eq!(msg_info.count(), 1);
                    let c = get_mr(0) as u8;
                    self.device.put_char(c);
                    MessageInfo::new(0, 0)
                }
                1 => {
                    assert_eq!(msg_info.count(), 0);
                    match self.buffer.pop_front() {
                        None => {
                            self.notify = true;
                            MessageInfo::new(0, 0)
                        }
                        Some(c) => {
                            set_mr(0, c as MessageValue);
                            MessageInfo::new(1, 1)
                        }
                    }
                }
                _ => {
                    panic!()
                }
            },
            _ => {
                unreachable!()
            }
        })
    }
}
