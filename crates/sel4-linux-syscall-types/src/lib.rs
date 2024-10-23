//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

// TODO use https://github.com/mebeim/linux-syscalls/tree/master/db

#![no_std]
#![feature(c_variadic)]
// #![allow(non_snake_case)]
// #![allow(non_upper_case_globals)]

use core::ffi::c_void;

mod arch;
mod syscall_registers;

pub use arch::*;
pub use syscall_registers::{
    IteratorAsSyscallRegisters, SyscallRegisterValue, SyscallRegisterWord, SyscallRegisters,
    VaListAsSyscallRegisters,
};

pub const ENOSYS: i64 = 38;
pub const ENOMEM: i64 = 12;

pub const SEEK_CUR: i32 = 1;
pub const MAP_ANONYMOUS: i32 = 0x20;

#[repr(C)]
#[derive(Debug)]
pub struct IOVec {
    pub iov_base: *const c_void,
    pub iov_len: usize,
}

#[derive(Debug)]
pub enum Syscall {
    Lseek {
        fd: i32,
        offset: usize,
        whence: i32,
    },
    Write {
        fd: i32,
        buf: *const u8, // TODO c_void
        count: usize,
    },
    Writev {
        fd: i32,
        iov: *const IOVec,
        iovcnt: i32,
    },
    Getuid,
    Geteuid,
    Getgid,
    Getegid,
    Brk {
        addr: *const u8, // TODO c_void
    },
    Mmap {
        addr: *const u8, // TODO c_void
        length: usize,
        prot: i32,
        flag: i32,
        fd: i32,
        offset: usize,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParseSyscallError {
    UnrecognizedSyscallNumber,
    TooFewValues,
}

impl Syscall {
    pub fn parse(
        sysnum: isize,
        mut args: impl SyscallRegisters,
    ) -> Result<Self, ParseSyscallError> {
        use Syscall::*;

        fn next<T: SyscallRegisterValue>(
            args: &mut impl SyscallRegisters,
        ) -> Result<T, ParseSyscallError> {
            args.next_register_value()
                .ok_or(ParseSyscallError::TooFewValues)
        }

        let args = &mut args;

        Ok(match sysnum {
            syscall_number::LSEEK => Lseek {
                fd: next(args)?,
                offset: next(args)?,
                whence: next(args)?,
            },
            syscall_number::WRITE => Write {
                fd: next(args)?,
                buf: next(args)?,
                count: next(args)?,
            },
            syscall_number::WRITEV => Writev {
                fd: next(args)?,
                iov: next(args)?,
                iovcnt: next(args)?,
            },
            syscall_number::GETUID => Getuid,
            syscall_number::GETEUID => Geteuid,
            syscall_number::GETGID => Getgid,
            syscall_number::GETEGID => Getegid,
            syscall_number::BRK => Brk { addr: next(args)? },
            syscall_number::MMAP => Mmap {
                addr: next(args)?,
                length: next(args)?,
                prot: next(args)?,
                flag: next(args)?,
                fd: next(args)?,
                offset: next(args)?,
            },
            _ => return Err(ParseSyscallError::UnrecognizedSyscallNumber),
        })
    }
}
