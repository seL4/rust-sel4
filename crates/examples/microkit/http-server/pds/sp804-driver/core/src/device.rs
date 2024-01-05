//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

// TODO *const vs *mut

use core::ops::Deref;

use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};
use tock_registers::registers::{ReadOnly, ReadWrite, WriteOnly};
use tock_registers::{register_bitfields, register_structs};

register_structs! {
    #[allow(non_snake_case)]
    pub TimerRegisterBlock {
        (0x00 => Load: ReadWrite<u32>),
        (0x04 => Value: ReadWrite<u32>),
        (0x08 => Control: ReadWrite<u32, Control::Register>),
        (0x0c => IntClr: WriteOnly<u32>),
        (0x10 => RIS: ReadOnly<u32, RIS::Register>),
        (0x14 => MIS: ReadOnly<u32, MIS::Register>),
        (0x18 => BGLoad: ReadWrite<u32>),
        (0x1c => _reserved0),
        (0x20 => @END),
    }
}

register_bitfields! {
    u32,

    pub Control [
        TimerEn OFFSET(7) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1,
        ],
        TimerMode OFFSET(6) NUMBITS(1) [
            FreeRunning = 0,
            Periodic = 1,
        ],
        IntEnable OFFSET(5) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1,
        ],
        TimerPre OFFSET(2) NUMBITS(2) [
            Div1 = 0b00,
            Div16 = 0b01,
            Div256 = 0b10
        ],
        TimerSize OFFSET(1) NUMBITS(1) [
            Use16Bit = 0,
            Use32Bit = 1,
        ],
        OneShot OFFSET(0) NUMBITS(1) [
            Wrapping = 0,
            OneShot = 1,
        ]
    ],

    RIS [
        RIS OFFSET(0) NUMBITS(1) [],
    ],

    MIS [
        MIS OFFSET(0) NUMBITS(1) [],
    ],
}

pub struct Device {
    timer_1: Timer,
    timer_2: Timer,
}

impl Device {
    pub unsafe fn new(ptr: *const ()) -> Self {
        let ptr = ptr.cast::<TimerRegisterBlock>();
        Device {
            timer_1: Timer::new(ptr.offset(0)),
            timer_2: Timer::new(ptr.offset(1)),
        }
    }

    pub fn timer_1(&self) -> &Timer {
        &self.timer_1
    }

    pub fn timer_2(&self) -> &Timer {
        &self.timer_2
    }
}

pub struct Timer {
    ptr: *const TimerRegisterBlock,
}

#[allow(dead_code)]
impl Timer {
    pub unsafe fn new(ptr: *const TimerRegisterBlock) -> Self {
        Self { ptr }
    }

    fn ptr(&self) -> *const TimerRegisterBlock {
        self.ptr
    }

    pub fn get_load(&self) -> u32 {
        self.Load.get()
    }

    pub fn set_load(&self, value: u32) {
        self.Load.set(value)
    }

    pub fn current_value(&self) -> u32 {
        self.Value.get()
    }

    pub fn control(&self) -> &ReadWrite<u32, Control::Register> {
        &self.Control
    }

    pub fn set_free_running_mode(&self) {
        self.Control
            .modify(Control::TimerMode::FreeRunning + Control::OneShot::Wrapping)
    }

    pub fn set_periodic_mode(&self) {
        self.Control
            .modify(Control::TimerMode::Periodic + Control::OneShot::Wrapping)
    }

    pub fn set_one_shot_mode(&self) {
        self.Control.modify(Control::OneShot::OneShot)
    }

    pub fn set_backgroun_load(&self, value: u32) {
        self.BGLoad.set(value)
    }

    pub fn clear_interrupt(&self) {
        let value = 1; // arbitrary
        self.IntClr.set(value);
    }

    pub fn raw_interrupt_status(&self) -> bool {
        self.RIS.read(RIS::RIS) != 0
    }

    pub fn masked_interrupt_status(&self) -> bool {
        self.MIS.read(MIS::MIS) != 0
    }
}

impl Deref for Timer {
    type Target = TimerRegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}
