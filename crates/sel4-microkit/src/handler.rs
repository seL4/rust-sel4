//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::fmt;

use sel4_microkit_base::MessageInfo;

use crate::{
    defer::{DeferredAction, PreparedDeferredAction},
    pd_is_passive, Channel,
};

pub use core::convert::Infallible;

const INPUT_CAP: sel4::Endpoint = sel4::Cap::from_bits(1);
const REPLY_CAP: sel4::Reply = sel4::Cap::from_bits(4);
const MONITOR_EP_CAP: sel4::Endpoint = sel4::Cap::from_bits(5);

const EVENT_TYPE_MASK: sel4::Word = 1 << (sel4::WORD_SIZE - 1);

/// Trait for the application-specific part of a protection domain's main loop.
pub trait Handler {
    /// Error type returned by this protection domain's entrypoints.
    type Error: fmt::Display;

    /// This method has the same meaning and type as its analog in `libmicrokit`.
    ///
    /// The default implementation just panics.
    fn notified(&mut self, channel: Channel) -> Result<(), Self::Error> {
        panic!("unexpected notification from channel {channel:?}")
    }

    /// This method has the same meaning and type as its analog in `libmicrokit`.
    ///
    /// The default implementation just panics.
    fn protected(
        &mut self,
        channel: Channel,
        msg_info: MessageInfo,
    ) -> Result<MessageInfo, Self::Error> {
        panic!("unexpected protected procedure call from channel {channel:?} with msg_info={msg_info:?}")
    }

    /// An advanced feature for use by protection domains which seek to coalesce syscalls when
    /// possible.
    ///
    /// This method is used by the main loop to fuse a queued `seL4_Send` call with the next
    /// `seL4_Recv` using `seL4_NBSendRecv`. Its default implementation just returns `None`.
    fn take_deferred_action(&mut self) -> Option<DeferredAction> {
        None
    }
}

pub(crate) enum Never {}

pub(crate) fn run_handler<T: Handler>(mut handler: T) -> Result<Never, T::Error> {
    let mut reply_tag: Option<MessageInfo> = None;

    let mut prepared_deferred_action: Option<PreparedDeferredAction> = if pd_is_passive() {
        sel4::with_ipc_buffer_mut(|ipc_buffer| ipc_buffer.msg_regs_mut()[0] = 0);
        Some(PreparedDeferredAction::new(
            MONITOR_EP_CAP.cast(),
            sel4::MessageInfoBuilder::default().length(1).build(),
        ))
    } else {
        None
    };

    loop {
        let (tag, badge) = match (reply_tag.take(), prepared_deferred_action.take()) {
            (Some(tag), None) => INPUT_CAP.reply_recv(tag.into_inner(), REPLY_CAP),
            (None, Some(action)) => action.cptr().nb_send_recv(
                action.msg_info(),
                INPUT_CAP.cast::<sel4::cap_type::Unspecified>(),
                REPLY_CAP,
            ),
            (None, None) => INPUT_CAP.recv(REPLY_CAP),
            _ => unreachable!(),
        };

        let tag = MessageInfo::from_inner(tag);

        let is_endpoint = badge & EVENT_TYPE_MASK != 0;

        if is_endpoint {
            let channel_index = badge & (sel4::Word::try_from(sel4::WORD_SIZE).unwrap() - 1);
            reply_tag =
                Some(handler.protected(Channel::new(channel_index.try_into().unwrap()), tag)?);
        } else {
            let mut badge_bits = badge;
            while badge_bits != 0 {
                let i = badge_bits.trailing_zeros();
                handler.notified(Channel::new(i.try_into().unwrap()))?;
                badge_bits &= !(1 << i);
            }
        };

        prepared_deferred_action = handler
            .take_deferred_action()
            .as_ref()
            .map(DeferredAction::prepare);

        if prepared_deferred_action.is_some() && is_endpoint {
            panic!("handler yielded deferred action after call to 'protected()'");
        }
    }
}

/// A [`Handler`] implementation which does not override any of the default method implementations.
pub struct NullHandler(());

impl NullHandler {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(())
    }
}

impl Handler for NullHandler {
    type Error = Infallible;
}
