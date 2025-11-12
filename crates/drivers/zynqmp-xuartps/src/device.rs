#![allow(dead_code)]

use core::ops::Deref;

use tock_registers::{
    interfaces::{Readable, Writeable},
    register_structs,
    registers::{ReadOnly, ReadWrite},
};

const CHANNEL_STS_TX_EMPTY: u32 = 1 << 3;

register_structs! {
    #[allow(non_snake_case)]
    pub(crate) RegisterBlock {
        (0x000 => Control: ReadWrite<u32>),
        (0x004 => Mode: ReadWrite<u32>),
        (0x008 => Intpt_en: ReadWrite<u32>),
        (0x00C => Intrp_dis: ReadWrite<u32>),
        (0x010 => Intrp_mask: ReadWrite<u32>),
        (0x014 => Chnl_int_sts: ReadWrite<u32>),
        (0x018 => Baud_rate_gen: ReadWrite<u32>),
        (0x01C => Rcvr_timeout: ReadWrite<u32>),
        (0x020 => Rcvr_FIFO_trigger_level: ReadWrite<u32>),
        (0x024 => Modem_ctrl: ReadWrite<u32>),
        (0x028 => Modem_sts: ReadWrite<u32>),
        (0x02C => Channel_sts: ReadOnly<u32>),
        (0x030 => TX_RX_FIFO: ReadWrite<u32>),
        (0x034 => Baud_rate_divider: ReadWrite<u32>),
        (0x038 => Flow_delay: ReadWrite<u32>),
        (0x03C => _reserved0),
        (0x044 => Tx_FIFO_trigger_level: ReadWrite<u32>),
        (0x048 => Rx_FIFO_byte_status: ReadWrite<u32>),
        (0x04C => @END),
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
            if self.Channel_sts.get() & CHANNEL_STS_TX_EMPTY != 0 {
                break;
            }
        }
        self.TX_RX_FIFO.set(c as u32);
    }
}
