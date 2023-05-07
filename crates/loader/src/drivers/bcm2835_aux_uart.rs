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
    pub Bcm2835AuxUartRegisterBlock {
        (0x000 => _reserved0),
        (0x040 => IO: ReadWrite<u8>),
        (0x041 => _reserved1),
        (0x044 => IER: ReadWrite<u32>),
        (0x048 => _reserved2),
        (0x054 => LSR: ReadWrite<u32>),
        (0x058 => @END),
    }
}

pub struct Bcm2835AuxUartDevice {
    base_addr: usize,
}

impl Bcm2835AuxUartDevice {
    pub const unsafe fn new(base_addr: usize) -> Self {
        Self { base_addr }
    }

    fn ptr(&self) -> *const Bcm2835AuxUartRegisterBlock {
        self.base_addr as *const _
    }

    pub fn init(&self) {}
}

impl Deref for Bcm2835AuxUartDevice {
    type Target = Bcm2835AuxUartRegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}

impl Bcm2835AuxUartDevice {
    pub fn put_char(&self, c: u8) {
        loop {
            if self.LSR.get() & MU_LSR_TXIDLE != 0 {
                break;
            }
        }
        self.IO.set(c);
    }
}
