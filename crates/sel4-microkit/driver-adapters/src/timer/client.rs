//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::time::Duration;

use sel4_driver_interfaces::timer::{Clock, ErrorType, NumTimers, Timers};
use sel4_microkit::Channel;
use sel4_microkit_simple_ipc as simple_ipc;

use super::message_types::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Client {
    channel: Channel,
}

impl Client {
    pub fn new(channel: Channel) -> Self {
        Client { channel }
    }

    fn request(&self, req: Request) -> Result<SuccessResponse, Error> {
        simple_ipc::call::<_, Response>(self.channel, req)
            .map_err(|_| Error::InvalidResponse)?
            .map_err(Error::ErrorResponse)
    }
}

impl ErrorType for Client {
    type Error = Error;
}

impl Clock for Client {
    fn get_time(&mut self) -> Result<Duration, Self::Error> {
        match self.request(Request::GetTime)? {
            SuccessResponse::GetTime(v) => Ok(v),
            _ => Err(Error::UnexpectedResponse),
        }
    }
}

impl Timers for Client {
    type TimerLayout = NumTimers;

    type Timer = usize;

    fn timer_layout(&mut self) -> Result<Self::TimerLayout, Self::Error> {
        match self.request(Request::NumTimers)? {
            SuccessResponse::NumTimers(v) => Ok(NumTimers(v)),
            _ => Err(Error::UnexpectedResponse),
        }
    }

    fn set_timeout_on(
        &mut self,
        timer: Self::Timer,
        relative: Duration,
    ) -> Result<(), Self::Error> {
        match self.request(Request::SetTimeout { timer, relative })? {
            SuccessResponse::SetTimeout => Ok(()),
            _ => Err(Error::UnexpectedResponse),
        }
    }

    fn clear_timeout_on(&mut self, timer: Self::Timer) -> Result<(), Self::Error> {
        match self.request(Request::ClearTimeout { timer })? {
            SuccessResponse::ClearTimeout => Ok(()),
            _ => Err(Error::UnexpectedResponse),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Error {
    ErrorResponse(ErrorResponse),
    InvalidResponse,
    UnexpectedResponse,
}
