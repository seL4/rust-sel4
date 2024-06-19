//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![allow(dead_code)]

use core::ops::Deref;

use tock_registers::{
    interfaces::{Readable, Writeable},
    register_structs,
    registers::ReadWrite,
};

const MU_LSR_TXIDLE: u32 = 1 << 6;
const MU_LSR_DATAREADY: u32 = 1 << 0;

register_structs! {
    #[allow(non_snake_case)]
    pub(crate) RegisterBlock {
        (0x000 => _reserved0),
        (0x040 => IO: ReadWrite<u8>),
        (0x041 => _reserved1),
        (0x044 => IER: ReadWrite<u32>),
        (0x048 => _reserved2),
        (0x054 => LSR: ReadWrite<u32>),
        (0x058 => @END),
    }
}

pub(crate) struct Device {
    ptr: *mut RegisterBlock,
}

impl Device {
    pub(crate) const unsafe fn new(ptr: *mut RegisterBlock) -> Self {
        Self { ptr }
    }

    fn ptr(&self) -> *const RegisterBlock {
        self.ptr
    }

    pub(crate) fn init(&self) {}
}

impl Deref for Device {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}

impl Device {
    pub(crate) fn put_char(&self, c: u8) {
        loop {
            if self.LSR.get() & MU_LSR_TXIDLE != 0 {
                break;
            }
        }
        self.IO.set(c);
    }
}
