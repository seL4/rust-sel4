#![allow(dead_code)]

use core::ops::Deref;

use tock_registers::{
    interfaces::{Readable, Writeable},
    register_structs,
    registers::ReadWrite,
};

const PL011_UARTFR_TXFF: u32 = 1 << 5;
const PL011_UARTFR_RXFE: u32 = 1 << 4;

register_structs! {
    #[allow(non_snake_case)]
    pub(crate)Pl011RegisterBlock {
        (0x000 => DR: ReadWrite<u8>),
        (0x001 => _reserved0),
        (0x018 => FR: ReadWrite<u32>),
        (0x01c => _reserved1),
        (0x038 => IMSC: ReadWrite<u32>),
        (0x03c => _reserved2),
        (0x044 => ICR: ReadWrite<u32>),
        (0x048 => @END),
    }
}

pub(crate) struct Pl011Device {
    base_addr: usize,
}

impl Pl011Device {
    pub(crate) const unsafe fn new(base_addr: usize) -> Self {
        Self { base_addr }
    }

    fn ptr(&self) -> *const Pl011RegisterBlock {
        self.base_addr as *const _
    }

    pub(crate) fn init(&self) {
        self.IMSC.set(0x50);
    }
}

impl Deref for Pl011Device {
    type Target = Pl011RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}

impl Pl011Device {
    pub(crate) fn put_char(&self, c: u8) {
        loop {
            if self.FR.get() & PL011_UARTFR_TXFF == 0 {
                break;
            }
        }
        self.DR.set(c)
    }
}
