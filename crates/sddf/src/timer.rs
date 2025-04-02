//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![allow(unused_variables)]
#![allow(dead_code)]

use core::convert::Infallible;

use num_enum::{IntoPrimitive, TryFromPrimitive};

use sddf_sys as sys;
use sel4_microkit_message_types::*;

use crate::{common::*, Config};

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

impl MessageSend for TimerRequest {
    type Label = TimerRequestMessageLabel;

    type Error = Infallible;

    fn write_message(
        &self,
        regs: MessageRegistersMut,
    ) -> Result<(Self::Label, MessageRegistersPrefixLength), Self::Error> {
        regs.with_words_around_label(|buf| {
            Ok(match self {
                Self::GetTime => (TimerRequestMessageLabel::GetTime, 0),
                Self::SetTimeout { nanoseconds } => {
                    buf[0] = *nanoseconds;
                    (TimerRequestMessageLabel::SetTimeout, 1)
                }
            })
        })
    }
}

impl MessageRecv for TimerRequest {
    type Label = TimerRequestMessageLabel;

    type Error = PeerMisbehaviorError;

    fn read_message(label: Self::Label, regs: &MessageRegisters) -> Result<Self, Self::Error> {
        Ok(match label {
            Self::Label::GetTime => Self::GetTime,
            Self::Label::SetTimeout => {
                let nanoseconds = regs.as_words().get(0).ok_or(PeerMisbehaviorError::new())?;
                Self::SetTimeout {
                    nanoseconds: *nanoseconds,
                }
            }
        })
    }
}
