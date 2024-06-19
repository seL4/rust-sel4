//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::convert::Infallible;

use sel4_driver_interfaces::timer::{NumTimers, Timers};
use sel4_driver_interfaces::HandleInterrupt;
use sel4_microkit::{Channel, Handler, MessageInfo};
use sel4_microkit_message::MessageInfoExt;

use super::message_types::*;

#[derive(Clone, Debug)]
pub struct HandlerImpl<Driver> {
    driver: Driver,
    timer: Channel,
    client: Channel,
    num_timers: usize,
}

impl<Driver: Timers<TimerLayout = NumTimers>> HandlerImpl<Driver> {
    pub fn new(mut driver: Driver, timer: Channel, client: Channel) -> Result<Self, Driver::Error> {
        let num_timers = driver.timer_layout()?.0;
        Ok(Self {
            driver,
            timer,
            client,
            num_timers,
        })
    }

    fn guard_timer(&self, timer: usize) -> Result<(), ErrorResponse> {
        if timer < self.num_timers {
            Ok(())
        } else {
            Err(ErrorResponse::TimerOutOfBounds)
        }
    }
}

impl<Driver> Handler for HandlerImpl<Driver>
where
    Driver: Timers<TimerLayout = NumTimers, Timer = usize> + HandleInterrupt,
{
    type Error = Infallible;

    fn notified(&mut self, channel: Channel) -> Result<(), Self::Error> {
        if channel == self.timer {
            self.driver.handle_interrupt();
            self.timer.irq_ack().unwrap();
            self.client.notify();
        } else {
            panic!("unexpected channel: {channel:?}");
        }
        Ok(())
    }

    fn protected(
        &mut self,
        channel: Channel,
        msg_info: MessageInfo,
    ) -> Result<MessageInfo, Self::Error> {
        if channel == self.client {
            Ok(match msg_info.recv_using_postcard::<Request>() {
                Ok(req) => {
                    let resp = match req {
                        Request::GetTime => self
                            .driver
                            .get_time()
                            .map(SuccessResponse::GetTime)
                            .map_err(|_| ErrorResponse::Unspecified),
                        Request::NumTimers => self
                            .driver
                            .timer_layout()
                            .map(|NumTimers(n)| SuccessResponse::NumTimers(n))
                            .map_err(|_| ErrorResponse::Unspecified),
                        Request::SetTimeout { timer, relative } => {
                            self.guard_timer(timer).and_then(|_| {
                                self.driver
                                    .set_timeout_on(timer, relative)
                                    .map(|_| SuccessResponse::SetTimeout)
                                    .map_err(|_| ErrorResponse::Unspecified)
                            })
                        }
                        Request::ClearTimeout { timer } => self.guard_timer(timer).and_then(|_| {
                            self.driver
                                .clear_timeout_on(timer)
                                .map(|_| SuccessResponse::ClearTimeout)
                                .map_err(|_| ErrorResponse::Unspecified)
                        }),
                    };
                    MessageInfo::send_using_postcard(resp).unwrap()
                }
                Err(_) => MessageInfo::send_unspecified_error(),
            })
        } else {
            panic!("unexpected channel: {channel:?}");
        }
    }
}
