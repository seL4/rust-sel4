//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![allow(unused_variables)]
#![allow(dead_code)]

use core::convert::Infallible;

use num_enum::{IntoPrimitive, TryFromPrimitive};

use sddf_ipc_types::*;
use sddf_sys as sys;

use crate::Config;

#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ClientConfig(sys::timer_client_config);

unsafe impl Config for ClientConfig {
    fn is_magic_valid(&self) -> bool {
        self.0.magic == sys::SDDF_TIMER_MAGIC
    }
}

impl ClientConfig {
    pub fn driver_id(&self) -> u8 {
        self.0.driver_id
    }
}

pub type Nanoseconds = MessageRegisterValue;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum TimerRequest {
    GetTime,
    SetTimeout { nanoseconds: Nanoseconds },
}

pub struct TimerSetTimeoutResponse;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct TimerNowResponse {
    nanoseconds: Nanoseconds,
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[cfg_attr(target_pointer_width = "32", repr(u32))]
#[cfg_attr(target_pointer_width = "64", repr(u64))]
pub enum TimerRequestMessageLabel {
    GetTime = 0,
    SetTimeout = 1,
}

impl MessageWriter for TimerRequest {
    type Error = Infallible;

    fn write_message(
        &self,
        buf: &mut [MessageRegisterValue],
    ) -> Result<(MessageLabel, usize), Self::Error> {
        let mut builder = MessageBuilder::new(buf);
        match self {
            Self::GetTime => {
                builder.set_label(TimerRequestMessageLabel::GetTime.into());
            }
            Self::SetTimeout { nanoseconds } => {
                builder.set_label(TimerRequestMessageLabel::SetTimeout.into());
                builder.set_next_mr(*nanoseconds);
            }
        }
        Ok(builder.build())
    }
}

impl ReadFromMessage for TimerRequest {
    type Error = MessagParseError;

    fn read_from_message(
        label: MessageLabel,
        buf: &[MessageRegisterValue],
    ) -> Result<Self, Self::Error> {
        let parser = MessageParser::new(label, buf);
        Ok(match parser.label_try_into()? {
            TimerRequestMessageLabel::GetTime => Self::GetTime,
            TimerRequestMessageLabel::SetTimeout => {
                let mut i = 0..;
                let nanoseconds = parser.get_mr(i.next().unwrap())?;
                Self::SetTimeout { nanoseconds }
            }
        })
    }
}
