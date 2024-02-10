//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ffi::c_int;

use super::{get_ipc_buffer, get_ipc_buffer_mut, get_ipc_buffer_ptr, set_ipc_buffer_ptr};
use crate::{seL4_CPtr, seL4_IPCBuffer, seL4_Word};

#[no_mangle]
pub unsafe extern "C" fn seL4_SetIPCBuffer(ipc_buffer_ptr: *mut seL4_IPCBuffer) {
    set_ipc_buffer_ptr(ipc_buffer_ptr)
}

#[no_mangle]
pub unsafe extern "C" fn seL4_GetIPCBuffer() -> *mut seL4_IPCBuffer {
    get_ipc_buffer_ptr()
}

#[no_mangle]
pub extern "C" fn seL4_GetMR(i: c_int) -> seL4_Word {
    get_ipc_buffer().get_mr(i.try_into().unwrap())
}

#[no_mangle]
pub extern "C" fn seL4_SetMR(i: c_int, value: seL4_Word) {
    get_ipc_buffer_mut().set_mr(i.try_into().unwrap(), value)
}

#[no_mangle]
pub extern "C" fn seL4_GetUserData() -> seL4_Word {
    get_ipc_buffer().userData
}

#[no_mangle]
pub extern "C" fn seL4_SetUserData(data: seL4_Word) {
    get_ipc_buffer_mut().userData = data;
}

#[no_mangle]
pub extern "C" fn seL4_GetBadge(i: c_int) -> seL4_Word {
    get_ipc_buffer().caps_or_badges[usize::try_from(i).unwrap()]
}

#[no_mangle]
pub extern "C" fn seL4_GetCap(i: c_int) -> seL4_CPtr {
    get_ipc_buffer().get_cap(i.try_into().unwrap())
}

#[no_mangle]
pub extern "C" fn seL4_SetCap(i: c_int, cap: seL4_CPtr) {
    get_ipc_buffer_mut().set_cap(i.try_into().unwrap(), cap)
}

#[no_mangle]
pub unsafe extern "C" fn seL4_GetCapReceivePath(
    receiveCNode: *mut seL4_CPtr,
    receiveIndex: *mut seL4_CPtr,
    receiveDepth: *mut seL4_Word,
) {
    let ipc_buffer = get_ipc_buffer();
    if !receiveCNode.is_null() {
        *receiveCNode = ipc_buffer.receiveCNode;
    }
    if !receiveIndex.is_null() {
        *receiveIndex = ipc_buffer.receiveIndex;
    }
    if !receiveDepth.is_null() {
        *receiveDepth = ipc_buffer.receiveDepth;
    }
}

#[no_mangle]
pub extern "C" fn seL4_SetCapReceivePath(
    receiveCNode: seL4_CPtr,
    receiveIndex: seL4_CPtr,
    receiveDepth: seL4_Word,
) {
    let ipc_buffer = get_ipc_buffer_mut();
    ipc_buffer.receiveCNode = receiveCNode;
    ipc_buffer.receiveIndex = receiveIndex;
    ipc_buffer.receiveDepth = receiveDepth;
}
