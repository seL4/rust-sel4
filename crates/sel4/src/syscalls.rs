use core::array;

use crate::{
    sys, Endpoint, IPCBuffer, InvocationContext, MessageInfo, Notification, Word,
    NUM_FAST_MESSAGE_REGISTERS,
};

pub type Badge = Word;

impl<C: InvocationContext> Endpoint<C> {
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
    pub fn recv(self) -> (MessageInfo, Badge) {
        let (raw_msg_info, badge) =
            self.invoke(|cptr, ipc_buffer| ipc_buffer.inner_mut().seL4_Recv(cptr.bits()));
        (MessageInfo::from_inner(raw_msg_info), badge)
    }

    /// Corresponds to `seL4_NBRecv`.
    pub fn nb_recv(self) -> (MessageInfo, Badge) {
        let (raw_msg_info, badge) =
            self.invoke(|cptr, ipc_buffer| ipc_buffer.inner_mut().seL4_NBRecv(cptr.bits()));
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

    pub fn recv_with_mrs(self) -> RecvWithMRs {
        let mut msg = [0; NUM_FAST_MESSAGE_REGISTERS];
        let [mr0, mr1, mr2, mr3] = msg.each_mut().map(Some);
        let (raw_msg_info, badge) = self.invoke(|cptr, ipc_buffer| {
            ipc_buffer
                .inner_mut()
                .seL4_RecvWithMRs(cptr.bits(), mr0, mr1, mr2, mr3)
        });
        RecvWithMRs {
            info: MessageInfo::from_inner(raw_msg_info),
            badge,
            msg,
        }
    }

    pub fn call_with_mrs<T: FastMessages>(self, info: MessageInfo, messages: T) -> CallWithMRs {
        let mut msg = messages.prepare_in_out();
        let [mr0, mr1, mr2, mr3] = msg.each_mut().map(Some);
        let raw_msg_info = self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_CallWithMRs(
                cptr.bits(),
                info.into_inner(),
                mr0,
                mr1,
                mr2,
                mr3,
            )
        });
        CallWithMRs {
            info: MessageInfo::from_inner(raw_msg_info),
            msg,
        }
    }
}

impl<C: InvocationContext> Notification<C> {
    /// Corresponds to `seL4_Signal`.
    pub fn signal(self) {
        self.invoke(|cptr, ipc_buffer| ipc_buffer.inner_mut().seL4_Signal(cptr.bits()))
    }

    /// Corresponds to `seL4_Wait`.
    pub fn wait(self) -> Word {
        self.invoke(|cptr, ipc_buffer| ipc_buffer.inner_mut().seL4_Wait(cptr.bits()))
    }
}

/// Corresponds to `seL4_Reply`.
pub fn reply(ipc_buffer: &mut IPCBuffer, info: MessageInfo) {
    ipc_buffer.inner_mut().seL4_Reply(info.into_inner())
}

/// Corresponds to `seL4_Yield`.
pub fn r#yield() {
    sys::seL4_Yield()
}

//

const UNUSED_FOR_IN: Word = 0;

pub struct RecvWithMRs {
    pub info: MessageInfo,
    pub badge: Badge,
    pub msg: [Word; NUM_FAST_MESSAGE_REGISTERS],
}

pub struct CallWithMRs {
    pub info: MessageInfo,
    pub msg: [Word; NUM_FAST_MESSAGE_REGISTERS],
}

pub trait FastMessages: FastMessagesSealed {}

impl<T: FastMessagesSealed> FastMessages for T {}

pub trait FastMessagesSealed: FastMessagesUnchecked {}

impl FastMessagesSealed for [Word; 0] {}
impl FastMessagesSealed for [Word; 1] {}
impl FastMessagesSealed for [Word; 2] {}
impl FastMessagesSealed for [Word; 3] {}
impl FastMessagesSealed for [Word; 4] {}

type ConcreteFastMessagesForIn = [Option<Word>; NUM_FAST_MESSAGE_REGISTERS];

type ConcreteFastMessagesForInOut = [Word; NUM_FAST_MESSAGE_REGISTERS];

pub trait FastMessagesUnchecked {
    fn prepare_in(self) -> ConcreteFastMessagesForIn;

    fn prepare_in_out(self) -> ConcreteFastMessagesForInOut;
}

impl<const N: usize> FastMessagesUnchecked for [Word; N] {
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

#[allow(dead_code)]
#[allow(non_upper_case_globals)]
mod __assertions {
    use super::*;

    const __assert_num_fast_message_registers: () = {
        assert!(NUM_FAST_MESSAGE_REGISTERS == 4);
    };
}
