use crate::cspace::{
    Channel, DeferredAction, PreparedDeferredAction, INPUT_CAP, MONITOR_EP_CAP, REPLY_CAP,
};
use crate::message::MessageInfo;
use crate::pd_is_passive;

pub trait Handler {
    type Error;

    fn notified(&mut self, channel: Channel) -> Result<(), Self::Error> {
        panic!("unexpected notification from channel {channel:?}")
    }

    fn protected(
        &mut self,
        channel: Channel,
        msg_info: MessageInfo,
    ) -> Result<MessageInfo, Self::Error> {
        panic!("unexpected protected procedure call from channel {channel:?} with msg_info={msg_info:?}")
    }

    fn take_deferred_action(&mut self) -> Option<DeferredAction> {
        None
    }
}

pub(crate) fn run_handler<T: Handler>(mut handler: T) -> Result<!, T::Error> {
    let mut reply_tag: Option<MessageInfo> = None;

    let mut prepared_deferred_action: Option<PreparedDeferredAction> = if pd_is_passive() {
        sel4::with_borrow_ipc_buffer_mut(|ipc_buffer| ipc_buffer.msg_regs_mut()[0] = 0);
        Some(PreparedDeferredAction::new(
            MONITOR_EP_CAP.cast(),
            sel4::MessageInfoBuilder::default().length(1).build(),
        ))
    } else {
        None
    };

    loop {
        let (tag, badge) = match (reply_tag.take(), prepared_deferred_action.take()) {
            (Some(tag), None) => INPUT_CAP.reply_recv(tag.into_sel4(), REPLY_CAP),
            (None, Some(action)) => action.cptr().nb_send_recv(
                action.msg_info(),
                INPUT_CAP.cast::<sel4::cap_type::Unspecified>(),
                REPLY_CAP,
            ),
            (None, None) => INPUT_CAP.recv(REPLY_CAP),
            _ => unreachable!(),
        };

        let tag = MessageInfo::from_sel4(tag);

        let is_endpoint = badge & (1 << (sel4::WORD_SIZE - 1)) != 0;

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
    }
}

pub struct NullHandler(());

impl NullHandler {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(())
    }
}

impl Handler for NullHandler {
    type Error = !;
}
