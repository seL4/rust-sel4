//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

// TODO use https://github.com/mebeim/linux-syscalls/tree/master/db

#![no_std]
#![feature(c_variadic)]

use core::ffi::{c_char, c_uint, c_ulong, c_void};

mod arch;
mod syscall_registers;

pub use arch::*;
pub use syscall_registers::{
    IteratorAsSyscallArgs, SyscallArg, SyscallArgs, SyscallWordArg, VaListAsSyscallArgs,
};

pub type SyscallNumber = isize;

pub const ENOSYS: i64 = 38;
pub const ENOMEM: i64 = 12;

pub const SEEK_CUR: i32 = 1;
pub const MAP_ANONYMOUS: i32 = 0x20;

#[allow(non_camel_case_types)]
type c_off_t = usize;

#[allow(non_camel_case_types)]
type c_size_t = usize;

#[repr(C)]
#[derive(Debug)]
pub struct IOVec {
    pub iov_base: *const c_void,
    pub iov_len: usize,
}

#[derive(Debug)]
pub enum Syscall {
    Lseek {
        fd: c_uint,
        offset: c_off_t,
        whence: c_uint,
    },
    Write {
        fd: c_uint,
        buf: *const c_char, // TODO c_void
        count: c_size_t,
    },
    Writev {
        fd: c_uint,
        iov: *const IOVec,
        iovcnt: c_ulong,
    },
    Getuid,
    Geteuid,
    Getgid,
    Getegid,
    Brk {
        addr: c_ulong,
    },
    Mmap {
        addr: c_ulong,
        len: c_ulong,
        prot: c_ulong,
        flag: c_ulong,
        fd: c_ulong,
        offset: c_ulong,
    },
}

impl Syscall {
    pub fn parse(sysnum: isize, mut args: impl SyscallArgs) -> Result<Self, ParseSyscallError> {
        fn next<T: SyscallArg>(args: &mut impl SyscallArgs) -> Result<T, ParseSyscallError> {
            args.next_arg().ok_or(ParseSyscallError::TooFewValues)
        }

        let args = &mut args;

        use syscall_number::*;
        use Syscall::*;

        Ok(match sysnum {
            LSEEK => Lseek {
                fd: next(args)?,
                offset: next(args)?,
                whence: next(args)?,
            },
            WRITE => Write {
                fd: next(args)?,
                buf: next(args)?,
                count: next(args)?,
            },
            WRITEV => Writev {
                fd: next(args)?,
                iov: next(args)?,
                iovcnt: next(args)?,
            },
            GETUID => Getuid,
            GETEUID => Geteuid,
            GETGID => Getgid,
            GETEGID => Getegid,
            BRK => Brk { addr: next(args)? },
            MMAP => Mmap {
                addr: next(args)?,
                len: next(args)?,
                prot: next(args)?,
                flag: next(args)?,
                fd: next(args)?,
                offset: next(args)?,
            },
            _ => return Err(ParseSyscallError::UnrecognizedSyscallNumber),
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParseSyscallError {
    UnrecognizedSyscallNumber,
    TooFewValues,
}
