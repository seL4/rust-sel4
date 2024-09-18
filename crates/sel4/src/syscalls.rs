//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use core::array;

use sel4_config::{sel4_cfg, sel4_cfg_if};

use crate::{
    cap, cap_type, const_helpers::u32_into_usize, sys, Cap, CapType, ConveysReplyAuthority,
    InvocationContext, MessageInfo, Word, NUM_FAST_MESSAGE_REGISTERS,
};

#[sel4_cfg(not(KERNEL_MCS))]
use crate::IpcBuffer;

/// Number of message registers in the IPC buffer.
pub const NUM_MESSAGE_REGISTERS: usize = u32_into_usize(sys::seL4_MsgLimits::seL4_MsgMaxLength);

/// A capability badge.
pub type Badge = Word;

/// Trait for [`CapType`]s which are used as targets of IPC syscalls.
pub trait IpcCapType: CapType {}

impl IpcCapType for cap_type::Notification {}

impl IpcCapType for cap_type::Endpoint {}

// HACK
impl IpcCapType for cap_type::Unspecified {}

sel4_cfg_if! {
    if #[sel4_cfg(KERNEL_MCS)] {
        pub type WaitMessageInfo = MessageInfo;

        fn wait_message_info_from_sys(info: sys::WaitMessageInfo) -> WaitMessageInfo {
            MessageInfo::from_inner(info)
        }
    } else {
        pub type WaitMessageInfo = ();

        fn wait_message_info_from_sys(info: sys::WaitMessageInfo) -> WaitMessageInfo {
            info
        }
    }
}

impl<C: InvocationContext> cap::Endpoint<C> {
    /// Corresponds to `seL4_Send`.
    pub fn send(self, info: MessageInfo) {
        self.invoke(|cptr, ipc_buffer| {
            ipc_buffer
                .inner_mut()
                .seL4_Send(cptr.bits(), info.into_inner())
        })
    }

    /// Corresponds to `seL4_NBSend`.
    pub fn nb_send(self, info: MessageInfo) {
        self.invoke(|cptr, ipc_buffer| {
            ipc_buffer
                .inner_mut()
                .seL4_NBSend(cptr.bits(), info.into_inner())
        })
    }

    /// Corresponds to `seL4_Recv`.
    pub fn recv(self, reply_authority: impl ConveysReplyAuthority) -> (MessageInfo, Badge) {
        let (raw_msg_info, badge) = self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_Recv(
                cptr.bits(),
                reply_authority
                    .into_reply_authority()
                    .into_sys_reply_authority(),
            )
        });
        (MessageInfo::from_inner(raw_msg_info), badge)
    }

    /// Corresponds to `seL4_NBRecv`.
    pub fn nb_recv(self, reply_authority: impl ConveysReplyAuthority) -> (MessageInfo, Badge) {
        let (raw_msg_info, badge) = self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_NBRecv(
                cptr.bits(),
                reply_authority
                    .into_reply_authority()
                    .into_sys_reply_authority(),
            )
        });
        (MessageInfo::from_inner(raw_msg_info), badge)
    }

    /// Corresponds to `seL4_Call`.
    pub fn call(self, info: MessageInfo) -> MessageInfo {
        MessageInfo::from_inner(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer
                .inner_mut()
                .seL4_Call(cptr.bits(), info.into_inner())
        }))
    }

    /// Corresponds to `seL4_ReplyRecv`.
    pub fn reply_recv(
        self,
        info: MessageInfo,
        reply_authority: impl ConveysReplyAuthority,
    ) -> (MessageInfo, Badge) {
        let (raw_msg_info, badge) = self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_ReplyRecv(
                cptr.bits(),
                info.into_inner(),
                reply_authority
                    .into_reply_authority()
                    .into_sys_reply_authority(),
            )
        });
        (MessageInfo::from_inner(raw_msg_info), badge)
    }

    pub fn send_with_mrs<T: FastMessages>(self, info: MessageInfo, messages: T) {
        let [msg0, msg1, msg2, msg3] = messages.prepare_in();
        self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_SendWithMRs(
                cptr.bits(),
                info.into_inner(),
                msg0,
                msg1,
                msg2,
                msg3,
            )
        })
    }

    pub fn recv_with_mrs(self, reply_authority: impl ConveysReplyAuthority) -> RecvWithMRs {
        let mut msg = [0; NUM_FAST_MESSAGE_REGISTERS];
        let [ref mut mr0, ref mut mr1, ref mut mr2, ref mut mr3] = &mut msg;
        let (raw_msg_info, badge) = self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_RecvWithMRs(
                cptr.bits(),
                Some(mr0),
                Some(mr1),
                Some(mr2),
                Some(mr3),
                reply_authority
                    .into_reply_authority()
                    .into_sys_reply_authority(),
            )
        });
        RecvWithMRs {
            info: MessageInfo::from_inner(raw_msg_info),
            badge,
            msg,
        }
    }

    pub fn call_with_mrs<T: FastMessages>(self, info: MessageInfo, messages: T) -> CallWithMRs {
        let mut msg = messages.prepare_in_out();
        let [ref mut mr0, ref mut mr1, ref mut mr2, ref mut mr3] = &mut msg;
        let raw_msg_info = self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_CallWithMRs(
                cptr.bits(),
                info.into_inner(),
                Some(mr0),
                Some(mr1),
                Some(mr2),
                Some(mr3),
            )
        });
        CallWithMRs {
            info: MessageInfo::from_inner(raw_msg_info),
            msg,
        }
    }
}

impl<C: InvocationContext> cap::Notification<C> {
    /// Corresponds to `seL4_Signal`.
    pub fn signal(self) {
        self.invoke(|cptr, ipc_buffer| ipc_buffer.inner_mut().seL4_Signal(cptr.bits()))
    }

    /// Corresponds to `seL4_Wait`.
    pub fn wait(self) -> (WaitMessageInfo, Badge) {
        let (info, badge) =
            self.invoke(|cptr, ipc_buffer| ipc_buffer.inner_mut().seL4_Wait(cptr.bits()));
        (wait_message_info_from_sys(info), badge)
    }
}

#[sel4_cfg(KERNEL_MCS)]
impl<C: InvocationContext> cap::Reply<C> {
    /// Corresponds to `seL4_Send`.
    pub fn send(self, info: MessageInfo) {
        self.invoke(|cptr, ipc_buffer| {
            ipc_buffer
                .inner_mut()
                .seL4_Send(cptr.bits(), info.into_inner())
        })
    }
}

// TODO more
impl<T: IpcCapType, C: InvocationContext> Cap<T, C> {
    /// Corresponds to `seL4_NBSendRecv`.
    #[sel4_cfg(KERNEL_MCS)]
    pub fn nb_send_recv<U: IpcCapType>(
        self,
        info: MessageInfo,
        src: Cap<U>,
        reply_authority: impl ConveysReplyAuthority,
    ) -> (MessageInfo, Badge) {
        let (raw_msg_info, badge) = self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_NBSendRecv(
                cptr.bits(),
                info.into_inner(),
                src.bits(),
                reply_authority
                    .into_reply_authority()
                    .into_sys_reply_authority(),
            )
        });
        (MessageInfo::from_inner(raw_msg_info), badge)
    }
}

/// Corresponds to `seL4_Reply`.
#[sel4_cfg(not(KERNEL_MCS))]
pub fn reply(ipc_buffer: &mut IpcBuffer, info: MessageInfo) {
    ipc_buffer.inner_mut().seL4_Reply(info.into_inner())
}

/// Corresponds to `seL4_Yield`.
pub fn r#yield() {
    sys::seL4_Yield()
}

/// Corresponds to `seL4_SetTLSBase`.
#[sel4_cfg(SET_TLS_BASE_SELF)]
pub fn set_tls_base(addr: usize) {
    sys::seL4_SetTLSBase(addr.try_into().unwrap())
}

//

const UNUSED_FOR_IN: Word = 0;

/// The result of [`cap::Endpoint::recv_with_mrs`].
pub struct RecvWithMRs {
    pub info: MessageInfo,
    pub badge: Badge,
    pub msg: [Word; NUM_FAST_MESSAGE_REGISTERS],
}

/// The result of [`cap::Endpoint::call_with_mrs`].
pub struct CallWithMRs {
    pub info: MessageInfo,
    pub msg: [Word; NUM_FAST_MESSAGE_REGISTERS],
}

type ConcreteFastMessagesForIn = [Option<Word>; NUM_FAST_MESSAGE_REGISTERS];

type ConcreteFastMessagesForInOut = [Word; NUM_FAST_MESSAGE_REGISTERS];

/// Trait for types which can hold the contents of a set of inline message registers.
pub trait FastMessages: fast_messages_sealing::FastMessagesSealed {
    fn prepare_in(self) -> ConcreteFastMessagesForIn;

    fn prepare_in_out(self) -> ConcreteFastMessagesForInOut;
}

impl<const N: usize> FastMessages for [Word; N]
where
    [Word; N]: fast_messages_sealing::FastMessagesSealed,
{
    fn prepare_in(self) -> ConcreteFastMessagesForIn {
        array::from_fn(|i| if i < self.len() { Some(self[i]) } else { None })
    }

    fn prepare_in_out(self) -> ConcreteFastMessagesForInOut {
        array::from_fn(|i| {
            if i < self.len() {
                self[i]
            } else {
                UNUSED_FOR_IN
            }
        })
    }
}

impl FastMessages for &[Word] {
    fn prepare_in(self) -> ConcreteFastMessagesForIn {
        assert!(self.len() <= NUM_FAST_MESSAGE_REGISTERS);
        array::from_fn(|i| if i < self.len() { Some(self[i]) } else { None })
    }

    fn prepare_in_out(self) -> ConcreteFastMessagesForInOut {
        assert!(self.len() <= NUM_FAST_MESSAGE_REGISTERS);
        array::from_fn(|i| {
            if i < self.len() {
                self[i]
            } else {
                UNUSED_FOR_IN
            }
        })
    }
}

mod fast_messages_sealing {
    use super::Word;

    pub trait FastMessagesSealed {}

    impl FastMessagesSealed for [Word; 0] {}
    impl FastMessagesSealed for [Word; 1] {}
    impl FastMessagesSealed for [Word; 2] {}
    impl FastMessagesSealed for [Word; 3] {}
    impl FastMessagesSealed for [Word; 4] {}

    impl FastMessagesSealed for &[Word] {}
}

#[allow(clippy::assertions_on_constants)]
const _: () = assert!(NUM_FAST_MESSAGE_REGISTERS == 4);
