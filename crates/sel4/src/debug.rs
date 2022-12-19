use core::ffi::c_char;

use crate::{sys, IPC_BUFFER, CapType, LocalCPtr, TCB};

pub fn debug_put_char(c: c_char) {
    sys::seL4_DebugPutChar(c)
}

pub fn debug_snapshot() {
    sys::seL4_DebugSnapshot()
}

impl TCB {
    pub fn debug_name(self, name: &[u8]) {
        sys::seL4_DebugNameThread(self.bits(), name, IPC_BUFFER.borrow_mut().as_mut().unwrap())
    }
}

impl<T: CapType> LocalCPtr<T> {
    pub fn debug_identify(self) -> u32 {
        sys::seL4_DebugCapIdentify(self.bits())
    }
}
