//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

// TODO use https://github.com/mebeim/linux-syscalls/tree/master/db

#![no_std]
#![feature(c_variadic)]

use core::ffi::{c_char, c_int, c_void};

mod arch;
mod syscall_registers;

pub use arch::*;
pub use syscall_registers::{
    IteratorAsSyscallArgs, SyscallArg, SyscallArgs, SyscallWordArg, VaListAsSyscallArgs,
};

pub type SyscallNumber = isize;

pub type SyscallReturnValue = isize;

pub const ENOSYS: SyscallReturnValue = 38;
pub const ENOMEM: SyscallReturnValue = 12;

pub const SEEK_CUR: i32 = 1;
pub const MAP_ANONYMOUS: i32 = 0x20;

#[allow(non_camel_case_types)]
type c_off_t = usize;

#[allow(non_camel_case_types)]
type c_size_t = usize;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct IOVec {
    pub iov_base: *const c_void,
    pub iov_len: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Syscall {
    Lseek {
        fd: c_int,
        offset: c_off_t,
        whence: c_int,
    },
    Write {
        fd: c_int,
        buf: *const c_char, // TODO c_void
        count: c_size_t,
    },
    Writev {
        fd: c_int,
        iov: *const IOVec,
        iovcnt: c_int,
    },
    Getuid,
    Geteuid,
    Getgid,
    Getegid,
    Brk {
        addr: *mut c_void,
    },
    Mmap {
        addr: *mut c_void,
        len: c_size_t,
        prot: c_int,
        flag: c_int,
        fd: c_int,
        offset: c_off_t,
    },
}

impl Syscall {
    pub fn parse<T: SyscallArgs>(
        sysnum: SyscallNumber,
        mut args: T,
    ) -> Result<Self, ParseSyscallError<T>> {
        Self::parse_inner(sysnum, &mut args).map_err(|err| match err {
            ParseSyscallErrorInner::UnrecognizedSyscallNumber => {
                ParseSyscallError::UnrecognizedSyscallNumber { sysnum, args }
            }
            ParseSyscallErrorInner::TooFewValues => ParseSyscallError::TooFewValues { sysnum },
        })
    }

    fn parse_inner<T: SyscallArgs>(
        sysnum: SyscallNumber,
        args: &mut T,
    ) -> Result<Self, ParseSyscallErrorInner> {
        fn next<T: SyscallArg>(args: &mut impl SyscallArgs) -> Result<T, ParseSyscallErrorInner> {
            args.next_arg().ok_or(ParseSyscallErrorInner::TooFewValues)
        }

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
            _ => return Err(ParseSyscallErrorInner::UnrecognizedSyscallNumber),
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParseSyscallError<T> {
    UnrecognizedSyscallNumber { sysnum: SyscallNumber, args: T },
    TooFewValues { sysnum: SyscallNumber },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParseSyscallErrorInner {
    UnrecognizedSyscallNumber,
    TooFewValues,
}
