use crate::{sys, Fault, MessageInfo, IPC_BUFFER};

impl Fault {
    pub fn get_from_ipc_buffer(info: &MessageInfo) -> Self {
        Self::from_sys(sys::seL4_Fault::get_from_ipc_buffer(
            info.inner(),
            &IPC_BUFFER.borrow().as_ref().unwrap(),
        ))
    }
}
