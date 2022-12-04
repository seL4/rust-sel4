use core::ffi::{c_char, CStr};

use sel4_config::sel4_cfg_if;

use super::ipc_buffer::get_ipc_buffer_mut;
use crate::{seL4_CPtr, seL4_MessageInfo, seL4_Uint32, seL4_Word};

macro_rules! ptr_to_opt {
    (
        $msg0:ident, $msg1:ident, $msg2:ident, $msg3:ident,
        $m0:ident, $m1:ident, $m2:ident, $m3:ident,
    ) => {
        $m0 = $msg0.as_ref().copied();
        $m1 = $msg1.as_ref().copied();
        $m2 = $msg2.as_ref().copied();
        $m3 = $msg3.as_ref().copied();
    };
}

macro_rules! ptr_to_opt_ref {
    (
        $msg0:ident, $msg1:ident, $msg2:ident, $msg3:ident,
        $m0:ident, $m1:ident, $m2:ident, $m3:ident,
    ) => {
        $m0 = $msg0.as_mut();
        $m1 = $msg1.as_mut();
        $m2 = $msg2.as_mut();
        $m3 = $msg3.as_mut();
    };
}

#[no_mangle]
pub unsafe extern "C" fn seL4_Send(dest: seL4_CPtr, msg_info: seL4_MessageInfo) {
    get_ipc_buffer_mut().seL4_Send(dest, msg_info)
}

#[no_mangle]
pub unsafe extern "C" fn seL4_SendWithMRs(
    dest: seL4_CPtr,
    msg_info: seL4_MessageInfo,
    msg0: *mut seL4_Word,
    msg1: *mut seL4_Word,
    msg2: *mut seL4_Word,
    msg3: *mut seL4_Word,
) {
    let m0;
    let m1;
    let m2;
    let m3;

    ptr_to_opt!(msg0, msg1, msg2, msg3, m0, m1, m2, m3,);

    crate::seL4_SendWithMRsWithoutIPCBuffer(dest, msg_info, m0, m1, m2, m3)
}

#[no_mangle]
pub unsafe extern "C" fn seL4_NBSend(dest: seL4_CPtr, msg_info: seL4_MessageInfo) {
    get_ipc_buffer_mut().seL4_NBSend(dest, msg_info)
}

#[no_mangle]
pub unsafe extern "C" fn seL4_NBSendWithMRs(
    dest: seL4_CPtr,
    msg_info: seL4_MessageInfo,
    msg0: *mut seL4_Word,
    msg1: *mut seL4_Word,
    msg2: *mut seL4_Word,
    msg3: *mut seL4_Word,
) {
    let m0;
    let m1;
    let m2;
    let m3;

    ptr_to_opt!(msg0, msg1, msg2, msg3, m0, m1, m2, m3,);

    crate::seL4_NBSendWithMRsWithoutIPCBuffer(dest, msg_info, m0, m1, m2, m3)
}

#[no_mangle]
pub unsafe extern "C" fn seL4_Reply(msg_info: seL4_MessageInfo) {
    get_ipc_buffer_mut().seL4_Reply(msg_info)
}

#[no_mangle]
pub unsafe extern "C" fn seL4_ReplyWithMRs(
    msg_info: seL4_MessageInfo,
    msg0: *mut seL4_Word,
    msg1: *mut seL4_Word,
    msg2: *mut seL4_Word,
    msg3: *mut seL4_Word,
) {
    let m0;
    let m1;
    let m2;
    let m3;

    ptr_to_opt!(msg0, msg1, msg2, msg3, m0, m1, m2, m3,);

    crate::seL4_ReplyWithMRsWithoutIPCBuffer(msg_info, m0, m1, m2, m3)
}

#[no_mangle]
pub unsafe extern "C" fn seL4_Signal(dest: seL4_CPtr) {
    get_ipc_buffer_mut().seL4_Signal(dest)
}

#[no_mangle]
pub unsafe extern "C" fn seL4_Recv(src: seL4_CPtr, sender: *mut seL4_Word) -> seL4_MessageInfo {
    let (msg_info, badge) = get_ipc_buffer_mut().seL4_Recv(src);

    if !sender.is_null() {
        *sender = badge;
    }

    msg_info
}

#[no_mangle]
pub unsafe extern "C" fn seL4_RecvWithMRs(
    src: seL4_CPtr,
    sender: *mut seL4_Word,
    msg0: *mut seL4_Word,
    msg1: *mut seL4_Word,
    msg2: *mut seL4_Word,
    msg3: *mut seL4_Word,
) -> seL4_MessageInfo {
    let m0;
    let m1;
    let m2;
    let m3;

    ptr_to_opt_ref!(msg0, msg1, msg2, msg3, m0, m1, m2, m3,);

    let (msg_info, badge) = crate::seL4_RecvWithMRsWithoutIPCBuffer(src, m0, m1, m2, m3);

    if !sender.is_null() {
        *sender = badge;
    }

    msg_info
}

#[no_mangle]
pub unsafe extern "C" fn seL4_NBRecv(src: seL4_CPtr, sender: *mut seL4_Word) -> seL4_MessageInfo {
    let (msg_info, badge) = get_ipc_buffer_mut().seL4_NBRecv(src);

    if !sender.is_null() {
        *sender = badge;
    }

    msg_info
}

#[no_mangle]
pub unsafe extern "C" fn seL4_Call(
    dest: seL4_CPtr,
    msg_info: seL4_MessageInfo,
) -> seL4_MessageInfo {
    get_ipc_buffer_mut().seL4_Call(dest, msg_info)
}

#[no_mangle]
pub unsafe extern "C" fn seL4_CallWithMRs(
    dest: seL4_CPtr,
    msg_info: seL4_MessageInfo,
    msg0: *mut seL4_Word,
    msg1: *mut seL4_Word,
    msg2: *mut seL4_Word,
    msg3: *mut seL4_Word,
) -> seL4_MessageInfo {
    let m0;
    let m1;
    let m2;
    let m3;

    ptr_to_opt_ref!(msg0, msg1, msg2, msg3, m0, m1, m2, m3,);

    let msg_info = crate::seL4_CallWithMRsWithoutIPCBuffer(dest, msg_info, m0, m1, m2, m3);

    msg_info
}

#[no_mangle]
pub unsafe extern "C" fn seL4_ReplyRecv(
    src: seL4_CPtr,
    msg_info: seL4_MessageInfo,
    sender: *mut seL4_Word,
) -> seL4_MessageInfo {
    let (out_msg_info, badge) = get_ipc_buffer_mut().seL4_ReplyRecv(src, msg_info);

    if !sender.is_null() {
        *sender = badge;
    }

    out_msg_info
}

#[no_mangle]
pub unsafe extern "C" fn seL4_Yield() {
    crate::seL4_Yield()
}

#[no_mangle]
pub unsafe extern "C" fn seL4_Wait(src: seL4_CPtr, sender: *mut seL4_Word) {
    let badge = get_ipc_buffer_mut().seL4_Wait(src);

    if !sender.is_null() {
        *sender = badge;
    }
}

#[no_mangle]
pub unsafe extern "C" fn seL4_Poll(src: seL4_CPtr, sender: *mut seL4_Word) -> seL4_MessageInfo {
    let (msg_info, badge) = get_ipc_buffer_mut().seL4_Poll(src);

    if !sender.is_null() {
        *sender = badge;
    }

    msg_info
}

sel4_cfg_if! {
    if #[cfg(DEBUG_BUILD)] {
        #[no_mangle]
        pub unsafe extern "C" fn seL4_DebugPutChar(c: c_char) {
            crate::seL4_DebugPutChar(c)
        }

        #[no_mangle]
        pub unsafe extern "C" fn seL4_DebugHalt(
        ) {
            crate::seL4_DebugHalt()
        }

        #[no_mangle]
        pub unsafe extern "C" fn seL4_DebugSnapshot(
        ) {
            crate::seL4_DebugSnapshot()
        }

        #[no_mangle]
        pub unsafe extern "C" fn seL4_DebugCapIdentify(cap: seL4_CPtr) -> seL4_Uint32 {
            crate::seL4_DebugCapIdentify(cap)
        }

        #[no_mangle]
        pub unsafe extern "C" fn seL4_DebugNameThread(tcb: seL4_CPtr, name: *const c_char) {
            crate::seL4_DebugNameThread(tcb, CStr::from_ptr(name).to_bytes(), &mut get_ipc_buffer_mut())
        }
    }
}

sel4_cfg_if! {
    if #[cfg(ENABLE_BENCHMARKS)] {

    }
}

sel4_cfg_if! {
    if #[cfg(SET_TLS_BASE_SELF)] {
        #[no_mangle]
        pub unsafe extern "C" fn seL4_SetTLSBase(tls_base: seL4_Word) {
            crate::seL4_SetTLSBase(tls_base)
        }
    }
}
