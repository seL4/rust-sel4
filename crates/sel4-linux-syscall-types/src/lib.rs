//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

// TODO use https://github.com/mebeim/linux-syscalls/tree/master/db

#![no_std]
#![feature(c_variadic)]

use core::ffi::{c_void, VaList};

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

impl Syscall {
    pub fn get(sysnum: i64, args: &mut VaList) -> Option<Self> {
        use Syscall::*;

        Some(match sysnum {
            62 => Lseek {
                fd: unsafe { args.arg() },
                offset: unsafe { args.arg() },
                whence: unsafe { args.arg() },
            },
            64 => Write {
                fd: unsafe { args.arg() },
                buf: unsafe { args.arg() },
                count: unsafe { args.arg() },
            },
            66 => Writev {
                fd: unsafe { args.arg() },
                iov: unsafe { args.arg() },
                iovcnt: unsafe { args.arg() },
            },
            174 => Getuid,
            175 => Geteuid,
            176 => Getgid,
            177 => Getegid,
            214 => Brk {
                addr: unsafe { args.arg() },
            },
            222 => Mmap {
                addr: unsafe { args.arg() },
                length: unsafe { args.arg() },
                prot: unsafe { args.arg() },
                flag: unsafe { args.arg() },
                fd: unsafe { args.arg() },
                offset: unsafe { args.arg() },
            },
            _ => return None,
        })
    }
}
