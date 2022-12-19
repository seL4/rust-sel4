use core::ffi::c_char;

use crate::{sys, InvocationContext, CapType, TCB, LocalCPtr};

pub fn debug_put_char(c: c_char) {
    sys::seL4_DebugPutChar(c)
}

pub fn debug_snapshot() {
    sys::seL4_DebugSnapshot()
}

impl<C: InvocationContext> TCB<C> {
    pub fn debug_name(self, name: &[u8]) {
        self.invoke(|cptr, ipc_buffer| {
            sys::seL4_DebugNameThread(cptr.bits(), name, ipc_buffer)
        })
    }
}

impl<T: CapType> LocalCPtr<T> {
    pub fn debug_identify(self) -> u32 {
        sys::seL4_DebugCapIdentify(self.bits())
    }
}
