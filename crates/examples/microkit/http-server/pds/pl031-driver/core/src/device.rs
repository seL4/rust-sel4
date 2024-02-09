//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

// TODO *const vs *mut

use core::ops::Deref;

use tock_registers::interfaces::Readable;
use tock_registers::registers::{ReadOnly, ReadWrite, WriteOnly};
use tock_registers::{register_bitfields, register_structs};

register_structs! {
    #[allow(non_snake_case)]
    pub RtcRegisterBlock {
        (0x000 => Data: ReadOnly<u32>),
        (0x004 => Match: ReadWrite<u32>),
        (0x008 => Load: ReadWrite<u32>),
        (0x00c => Control: ReadWrite<u32, Control::Register>),
        (0x010 => IMSC: ReadWrite<u32, IMSC::Register>),
        (0x014 => RIS: ReadOnly<u32, RIS::Register>),
        (0x018 => MIS: ReadOnly<u32, MIS::Register>),
        (0x01c => IC: WriteOnly<u32, IC::Register>),
        (0x020 => _reserved0),
        (0xffc => @END),
    }
}

register_bitfields! {
    u32,

    pub Control [
        Start OFFSET(0) NUMBITS(1) [],
    ],

    IMSC [
        IMSC OFFSET(0) NUMBITS(1) [],
    ],

    RIS [
        RIS OFFSET(0) NUMBITS(1) [],
    ],

    MIS [
        MIS OFFSET(0) NUMBITS(1) [],
    ],

    IC [
        IC OFFSET(0) NUMBITS(1) [],
    ],
}

pub struct Device {
    ptr: *const RtcRegisterBlock,
}

#[allow(dead_code)]
impl Device {
    pub unsafe fn new(ptr: *const ()) -> Self {
        let ptr = ptr.cast::<RtcRegisterBlock>();
        Self { ptr }
    }

    fn ptr(&self) -> *const RtcRegisterBlock {
        self.ptr
    }

    pub fn get_data(&self) -> u32 {
        self.Data.get()
    }
}

impl Deref for Device {
    type Target = RtcRegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}
