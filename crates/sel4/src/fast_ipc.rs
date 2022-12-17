#![allow(dead_code)]
#![allow(unused_imports)]

use core::array;

use crate::{
    local_cptr::*, sys, Badge, Endpoint, IPCBuffer, MessageInfo, Word, IPC_BUFFER,
    NUM_FAST_MESSAGE_REGISTERS,
};

const UNUSED_FOR_IN: Word = 0;

const __ASSERTION: [(); NUM_FAST_MESSAGE_REGISTERS] = [(); 4];

impl Endpoint {
    pub fn send_with_mrs<T: FastMessages>(&self, info: MessageInfo, messages: T) {
        let [msg0, msg1, msg2, msg3] = messages.prepare_in();
        IPC_BUFFER.borrow_mut().seL4_SendWithMRs(
            self.bits(),
            info.into_inner(),
            msg0,
            msg1,
            msg2,
            msg3,
        )
    }

    pub fn recv_with_mrs(&self) -> RecvWithMRs {
        let mut msg = [0; NUM_FAST_MESSAGE_REGISTERS];
        let [mr0, mr1, mr2, mr3] = msg.each_mut().map(Some);
        let (raw_msg_info, badge) =
            IPC_BUFFER
                .borrow_mut()
                .seL4_RecvWithMRs(self.bits(), mr0, mr1, mr2, mr3);
        RecvWithMRs {
            info: MessageInfo::from_inner(raw_msg_info),
            badge,
            msg,
        }
    }

    pub fn call_with_mrs<T: FastMessages>(&self, info: MessageInfo, messages: T) -> CallWithMRs {
        let mut msg = messages.prepare_in_out();
        let [mr0, mr1, mr2, mr3] = msg.each_mut().map(Some);
        let raw_msg_info = IPC_BUFFER.borrow_mut().seL4_CallWithMRs(
            self.bits(),
            info.into_inner(),
            mr0,
            mr1,
            mr2,
            mr3,
        );
        CallWithMRs {
            info: MessageInfo::from_inner(raw_msg_info),
            msg,
        }
    }
}

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
