use crate::{local_cptr::*, sys, Badge, IPCBuffer, InvocationContext, MessageInfo};

impl<C: InvocationContext> Endpoint<C> {
    pub fn send(self, info: MessageInfo) {
        self.invoke(|cptr, ipc_buffer| {
            ipc_buffer
                .inner_mut()
                .seL4_Send(cptr.bits(), info.into_inner())
        })
    }

    pub fn nb_send(self, info: MessageInfo) {
        self.invoke(|cptr, ipc_buffer| {
            ipc_buffer
                .inner_mut()
                .seL4_NBSend(cptr.bits(), info.into_inner())
        })
    }

    pub fn recv(self) -> (MessageInfo, Badge) {
        let (raw_msg_info, badge) =
            self.invoke(|cptr, ipc_buffer| ipc_buffer.inner_mut().seL4_Recv(cptr.bits()));
        (MessageInfo::from_inner(raw_msg_info), badge)
    }

    pub fn nb_recv(self) -> (MessageInfo, Badge) {
        let (raw_msg_info, badge) =
            self.invoke(|cptr, ipc_buffer| ipc_buffer.inner_mut().seL4_NBRecv(cptr.bits()));
        (MessageInfo::from_inner(raw_msg_info), badge)
    }

    pub fn call(self, info: MessageInfo) -> MessageInfo {
        MessageInfo::from_inner(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer
                .inner_mut()
                .seL4_Call(cptr.bits(), info.into_inner())
        }))
    }
}

impl<C: InvocationContext> Notification<C> {
    pub fn signal(self) {
        self.invoke(|cptr, ipc_buffer| ipc_buffer.inner_mut().seL4_Signal(cptr.bits()))
    }

    pub fn wait(self) -> Badge {
        self.invoke(|cptr, ipc_buffer| ipc_buffer.inner_mut().seL4_Wait(cptr.bits()))
    }
}

impl IPCBuffer {
    pub fn reply(&mut self, info: MessageInfo) {
        self.inner_mut().seL4_Reply(info.into_inner())
    }
}

pub fn r#yield() {
    sys::seL4_Yield()
}
