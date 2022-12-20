use crate::{sys, Fault, IPCBuffer, MessageInfo};

impl Fault {
    pub fn new(ipc_buffer: &IPCBuffer, info: &MessageInfo) -> Self {
        Self::from_sys(sys::seL4_Fault::get_from_ipc_buffer(
            info.inner(),
            ipc_buffer.inner(),
        ))
    }
}
