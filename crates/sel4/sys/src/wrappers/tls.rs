//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ptr;

use crate::seL4_IPCBuffer;

#[no_mangle]
#[thread_local]
static mut __sel4_ipc_buffer: *mut seL4_IPCBuffer = ptr::null_mut();

pub unsafe fn set_ipc_buffer_ptr(ptr: *mut seL4_IPCBuffer) {
    __sel4_ipc_buffer = ptr;
}

pub unsafe fn get_ipc_buffer_ptr() -> *mut seL4_IPCBuffer {
    __sel4_ipc_buffer
}

pub(crate) fn get_ipc_buffer() -> &'static seL4_IPCBuffer {
    unsafe {
        let ptr = get_ipc_buffer_ptr();
        assert!(!ptr.is_null());
        &*ptr
    }
}

pub(crate) fn get_ipc_buffer_mut() -> &'static mut seL4_IPCBuffer {
    unsafe {
        let ptr = get_ipc_buffer_ptr();
        assert!(!ptr.is_null());
        &mut *ptr
    }
}
