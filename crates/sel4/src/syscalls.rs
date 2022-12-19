use crate::{local_cptr::*, sys, Badge, MessageInfo, IPC_BUFFER};

// NOTE on the use of &self over self despite impl Copy for LocalCPtr.
// - LocalCPtr may someday include a reference to an IPC buffer and thus lose Copy.
// - &self enables convenient use of Deref at the cost of indirection.

impl Endpoint {
    pub fn send(&self, info: MessageInfo) {
        IPC_BUFFER
            .borrow_mut()
            .as_mut()
            .unwrap()
            .seL4_Send(self.bits(), info.into_inner())
    }

    pub fn nb_send(&self, info: MessageInfo) {
        IPC_BUFFER
            .borrow_mut()
            .as_mut()
            .unwrap()
            .seL4_NBSend(self.bits(), info.into_inner())
    }

    pub fn recv(&self) -> (MessageInfo, Badge) {
        let (raw_msg_info, badge) = IPC_BUFFER
            .borrow_mut()
            .as_mut()
            .unwrap()
            .seL4_Recv(self.bits());
        (MessageInfo::from_inner(raw_msg_info), badge)
    }

    pub fn nb_recv(&self) -> (MessageInfo, Badge) {
        let (raw_msg_info, badge) = IPC_BUFFER
            .borrow_mut()
            .as_mut()
            .unwrap()
            .seL4_NBRecv(self.bits());
        (MessageInfo::from_inner(raw_msg_info), badge)
    }

    pub fn call(&self, info: MessageInfo) -> MessageInfo {
        MessageInfo::from_inner(
            IPC_BUFFER
                .borrow_mut()
                .as_mut()
                .unwrap()
                .seL4_Call(self.bits(), info.into_inner()),
        )
    }
}

impl Notification {
    pub fn signal(&self) {
        IPC_BUFFER
            .borrow_mut()
            .as_mut()
            .unwrap()
            .seL4_Signal(self.bits());
    }

    pub fn wait(&self) -> Badge {
        IPC_BUFFER
            .borrow_mut()
            .as_mut()
            .unwrap()
            .seL4_Wait(self.bits())
    }
}

pub fn reply(info: MessageInfo) {
    IPC_BUFFER
        .borrow_mut()
        .as_mut()
        .unwrap()
        .seL4_Reply(info.into_inner())
}

pub fn r#yield() {
    sys::seL4_Yield()
}
