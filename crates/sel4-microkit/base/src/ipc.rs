//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::fmt;

use crate::{defer::PreparedDeferredAction, Channel, Child, MessageInfo};

const INPUT_CAP: sel4::cap::Endpoint = sel4::Cap::from_bits(1);
const REPLY_CAP: sel4::cap::Reply = sel4::Cap::from_bits(4);
const MONITOR_EP_CAP: sel4::cap::Endpoint = sel4::Cap::from_bits(5);

const IS_ENDPOINT_BADGE_BIT: usize = 63;
const IS_FAULT_BADGE_BIT: usize = 62;

fn strip_flag(badge: sel4::Badge, bit: usize) -> Option<sel4::Word> {
    let mask = 1 << bit;
    if badge & mask != 0 {
        Some(badge & !mask)
    } else {
        None
    }
}

#[doc(hidden)]
#[derive(Debug, Clone)]
pub enum Event {
    Notified(ChannelSet),
    Protected(Channel, MessageInfo),
    Fault(Child, MessageInfo),
}

impl Event {
    fn new(tag: sel4::MessageInfo, badge: sel4::Badge) -> Self {
        if let Some(channel_index) = strip_flag(badge, IS_ENDPOINT_BADGE_BIT) {
            Self::Protected(
                Channel::new(channel_index.try_into().unwrap()),
                MessageInfo::from_inner(tag),
            )
        } else if let Some(pd_index) = strip_flag(badge, IS_FAULT_BADGE_BIT) {
            Self::Fault(
                Child::new(pd_index.try_into().unwrap()),
                MessageInfo::from_inner(tag),
            )
        } else {
            Self::Notified(ChannelSet(badge))
        }
    }

    fn from_recv(recv: (sel4::MessageInfo, sel4::Badge)) -> Self {
        Self::new(recv.0, recv.1)
    }
}

#[doc(hidden)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChannelSet(sel4::Badge);

impl ChannelSet {
    pub fn contains(&self, channel: Channel) -> bool {
        (self.0 & (1 << channel.index())) != 0
    }

    pub fn iter(&self) -> ChannelSetIter {
        ChannelSetIter(self.0)
    }

    pub fn display(&self) -> DisplayChannelSet<'_> {
        DisplayChannelSet(self)
    }
}

#[doc(hidden)]
pub struct ChannelSetIter(sel4::Badge);

impl Iterator for ChannelSetIter {
    type Item = Channel;

    fn next(&mut self) -> Option<Self::Item> {
        let badge_bits = self.0;
        match badge_bits {
            0 => None,
            _ => {
                let i = badge_bits.trailing_zeros();
                self.0 = badge_bits & !(1 << i);
                Some(Channel::new(i.try_into().unwrap()))
            }
        }
    }
}

pub struct DisplayChannelSet<'a>(&'a ChannelSet);

impl fmt::Display for DisplayChannelSet<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{")?;
        let mut first = true;
        for channel in self.0.iter() {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{}", channel.index())?;
            first = false;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

pub fn reply(msg_info: MessageInfo) {
    REPLY_CAP.send(msg_info.into_inner())
}

pub fn recv() -> Event {
    Event::from_recv(INPUT_CAP.recv(REPLY_CAP))
}

pub fn reply_recv(msg_info: MessageInfo) -> Event {
    Event::from_recv(INPUT_CAP.reply_recv(msg_info.into_inner(), REPLY_CAP))
}

pub(crate) fn nb_send_recv(action: PreparedDeferredAction) -> Event {
    Event::from_recv(action.cptr().nb_send_recv(
        action.msg_info(),
        INPUT_CAP.cast::<sel4::cap_type::Unspecified>(),
        REPLY_CAP,
    ))
}

pub(crate) fn forfeit_sc() -> PreparedDeferredAction {
    PreparedDeferredAction::new(
        MONITOR_EP_CAP.cast(),
        sel4::MessageInfoBuilder::default().build(),
    )
}
