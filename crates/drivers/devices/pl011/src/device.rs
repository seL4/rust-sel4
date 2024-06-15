//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ops::Deref;

use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::registers::{ReadOnly, ReadWrite, WriteOnly};
use tock_registers::{register_bitfields, register_structs};

register_structs! {
    #[allow(non_snake_case)]
    pub RegisterBlock {
        (0x000 => DR: ReadWrite<u8>),
        (0x001 => _reserved0),
        (0x018 => FR: ReadOnly<u32, FR::Register>),
        (0x01c => _reserved1),
        (0x038 => IMSC: ReadWrite<u32, IMSC::Register>),
        (0x03c => _reserved2),
        (0x044 => ICR: WriteOnly<u32, ICR::Register>),
        (0x048 => @END),
    }
}

register_bitfields! {
    u32,

    FR [
        TXFF OFFSET(5) NUMBITS(1) [],
        RXFE OFFSET(4) NUMBITS(1) [],
    ],

    IMSC [
        RXIM OFFSET(4) NUMBITS(1) [],
    ],

    ICR [
        ALL OFFSET(0) NUMBITS(11) [],
    ],
}

pub struct Device {
    ptr: *mut RegisterBlock,
}

impl Device {
    pub unsafe fn new(ptr: *mut RegisterBlock) -> Self {
        Self { ptr }
    }

    fn ptr(&self) -> *mut RegisterBlock {
        self.ptr
    }

    pub fn init(&self) {
        self.IMSC.write(IMSC::RXIM::SET);
    }

    pub fn put_char(&self, c: u8) {
        while self.FR.matches_all(FR::TXFF::SET) {
            core::hint::spin_loop();
        }
        self.DR.set(c)
    }

    pub fn get_char(&self) -> Option<u8> {
        if self.FR.matches_all(FR::RXFE::CLEAR) {
            Some(self.DR.get())
        } else {
            None
        }
    }

    pub fn clear_all_interrupts(&self) {
        self.ICR.write(ICR::ALL::SET);
    }
}

impl Deref for Device {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}
