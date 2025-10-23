//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(c_variadic)]

use core::ffi::{CStr, c_char};

use sel4_immediate_sync_once_cell::ImmediateSyncOnceCell;

pub use sel4_linux_syscall_types::{
    ParseSyscallError, Syscall, SyscallReturnValue, VaListAsSyscallArgs,
};

// TODO: support the `struct __libc __libc;` state?

pub type SyscallHandler =
    fn(Result<Syscall, ParseSyscallError<VaListAsSyscallArgs>>) -> SyscallReturnValue;

pub type RawSyscallHandler = unsafe extern "C" fn(isize, ...) -> isize;

unsafe extern "C" {
    static mut __sysinfo: RawSyscallHandler;
    static mut __hwcap: usize;
    static mut __progname: *const c_char;
    static mut __progname_full: *const c_char;
}

static SYSCALL_HANDLER: ImmediateSyncOnceCell<SyscallHandler> = ImmediateSyncOnceCell::new();

unsafe extern "C" fn handle_syscall(sysnum: isize, mut args: ...) -> isize {
    (SYSCALL_HANDLER.get().unwrap())(Syscall::parse(sysnum, unsafe {
        VaListAsSyscallArgs::new(args.as_va_list())
    }))
}

#[allow(clippy::missing_safety_doc)]
pub unsafe fn set_syscall_handler(handler: SyscallHandler) {
    SYSCALL_HANDLER.set(handler).unwrap();
    set_raw_syscall_handler(handle_syscall);
}

#[allow(clippy::missing_safety_doc)]
pub unsafe fn set_raw_syscall_handler(handler: RawSyscallHandler) {
    __sysinfo = handler;
}

pub fn set_hwcap(hwcap: usize) {
    unsafe {
        __hwcap = hwcap;
    }
}

pub fn set_progname(s: &'static CStr) {
    unsafe {
        __progname = s.as_ptr();
    }
}

pub fn set_progname_full(s: &'static CStr) {
    unsafe {
        __progname_full = s.as_ptr();
    }
}
