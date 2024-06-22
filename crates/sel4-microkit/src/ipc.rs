//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_microkit_base::MessageInfo;

use crate::{defer::PreparedDeferredAction, Channel};

const INPUT_CAP: sel4::cap::Endpoint = sel4::Cap::from_bits(1);
const REPLY_CAP: sel4::cap::Reply = sel4::Cap::from_bits(4);
const MONITOR_EP_CAP: sel4::cap::Endpoint = sel4::Cap::from_bits(5);

const EVENT_TYPE_MASK: sel4::Word = 1 << (sel4::WORD_SIZE - 1);

#[doc(hidden)]
#[derive(Debug, Clone)]
pub enum Event {
    Protected(Channel, MessageInfo),
    Notified(NotifiedEvent),
}

impl Event {
    fn new(tag: sel4::MessageInfo, badge: sel4::Badge) -> Self {
        match badge & EVENT_TYPE_MASK {
            0 => Self::Notified(NotifiedEvent(badge)),
            _ => {
                let channel_index = badge & (sel4::Word::try_from(sel4::WORD_SIZE).unwrap() - 1);
                let channel = Channel::new(channel_index.try_into().unwrap());
                let tag = MessageInfo::from_inner(tag);
                Self::Protected(channel, tag)
            }
        }
    }

    fn from_recv(recv: (sel4::MessageInfo, sel4::Badge)) -> Self {
        Self::new(recv.0, recv.1)
    }
}

#[doc(hidden)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NotifiedEvent(sel4::Badge);

impl NotifiedEvent {
    pub fn iter(&self) -> NotifiedEventIter {
        NotifiedEventIter(self.0)
    }
}

#[doc(hidden)]
pub struct NotifiedEventIter(sel4::Badge);

impl Iterator for NotifiedEventIter {
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
        sel4::MessageInfoBuilder::default().length(1).build(),
    )
}
