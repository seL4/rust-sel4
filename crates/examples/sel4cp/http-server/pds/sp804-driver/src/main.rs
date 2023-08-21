#![no_std]
#![no_main]
#![feature(never_type)]

use core::time::Duration;

use sel4cp::message::{MessageInfo, NoMessageValue, StatusMessageLabel};
use sel4cp::{memory_region_symbol, protection_domain, var, Channel, Handler};

use sel4cp_http_server_example_sp804_driver_core::Driver;
use sel4cp_http_server_example_sp804_driver_interface_types::*;

const DEVICE: Channel = Channel::new(0);
const CLIENT: Channel = Channel::new(1);

#[protection_domain]
fn init() -> ThisHandler {
    let driver = unsafe {
        Driver::new(
            memory_region_symbol!(sp804_mmio_vaddr: *mut ()).as_ptr(),
            var!(freq: usize = 0).clone().try_into().unwrap(),
        )
    };
    ThisHandler { driver }
}

struct ThisHandler {
    driver: Driver,
}

impl Handler for ThisHandler {
    type Error = !;

    fn notified(&mut self, channel: Channel) -> Result<(), Self::Error> {
        match channel {
            DEVICE => {
                self.driver.handle_interrupt();
                DEVICE.irq_ack().unwrap();
                CLIENT.notify();
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
            CLIENT => match msg_info.label().try_into().ok() {
                Some(RequestTag::Now) => match msg_info.recv() {
                    Ok(NoMessageValue) => {
                        let now = self.driver.now();
                        MessageInfo::send(
                            StatusMessageLabel::Ok,
                            NowResponse {
                                micros: now.as_micros().try_into().unwrap(),
                            },
                        )
                    }
                    Err(_) => MessageInfo::send(StatusMessageLabel::Error, NoMessageValue),
                },
                Some(RequestTag::SetTimeout) => match msg_info.recv() {
                    Ok(SetTimeoutRequest { relative_micros }) => {
                        self.driver
                            .set_timeout(Duration::from_micros(relative_micros));
                        MessageInfo::send(StatusMessageLabel::Ok, NoMessageValue)
                    }
                    Err(_) => MessageInfo::send(StatusMessageLabel::Error, NoMessageValue),
                },
                Some(RequestTag::ClearTimeout) => match msg_info.recv() {
                    Ok(NoMessageValue) => {
                        self.driver.clear_timeout();
                        MessageInfo::send(StatusMessageLabel::Ok, NoMessageValue)
                    }
                    Err(_) => MessageInfo::send(StatusMessageLabel::Error, NoMessageValue),
                },
                None => MessageInfo::send(StatusMessageLabel::Error, NoMessageValue),
            },
            _ => {
                unreachable!()
            }
        })
    }
}
