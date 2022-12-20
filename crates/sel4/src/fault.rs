use crate::{sys, Fault, IPCBuffer, MessageInfo};

impl Fault {
    pub fn get_from_ipc_buffer(info: &MessageInfo, ipc_buffer: &IPCBuffer) -> Self {
        Self::from_sys(sys::seL4_Fault::get_from_ipc_buffer(
            info.inner(),
            ipc_buffer.inner(),
        ))
    }
}
