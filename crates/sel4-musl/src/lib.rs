//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(c_variadic)]

pub type SyscallHandler = unsafe extern "C" fn(i64, ...) -> i64;

extern "C" {
    static mut __sysinfo: SyscallHandler;
}

pub unsafe fn set_syscall_handler(handler: SyscallHandler) {
    __sysinfo = handler;
}
