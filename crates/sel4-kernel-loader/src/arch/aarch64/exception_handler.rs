//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::arch::asm;
use core::fmt;

use crate::arch::{Arch, ArchImpl};
use crate::fmt::debug_println_without_synchronization;

#[used]
#[no_mangle]
static mut exception_register_state: Registers = [0; NUM_REGISTERS];

#[no_mangle]
unsafe extern "C" fn exception_handler(vector_table_index: usize) {
    let mut esr;
    let mut far;
    let mut tpidr_el1;
    {
        asm!("mrs {}, esr_el2", out(reg) esr);
        asm!("mrs {}, far_el2", out(reg) far);
        asm!("mrs {}, tpidr_el1", out(reg) tpidr_el1);
    }
    let exception = Exception {
        vector_table_index,
        esr,
        far,
        tpidr_el1,
        registers: unsafe { exception_register_state },
    };
    debug_println_without_synchronization!("!!! Exception:\n{}", exception);
    ArchImpl::idle()
}

//

const NUM_REGISTERS: usize = 32;

type Registers = [u64; NUM_REGISTERS];

struct Exception {
    vector_table_index: usize,
    esr: usize,
    far: usize,
    tpidr_el1: usize,
    registers: Registers,
}

impl fmt::Display for Exception {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "Vector table index: {}",
            show_vector_table_index(self.vector_table_index).unwrap_or("<corrupted>")
        )?;
        writeln!(f, "ESR: 0x{:016x}", self.esr)?;
        writeln!(f, "FSR: 0x{:016x}", self.far)?;
        writeln!(f, "TPIDR_EL1: 0x{:016x}", self.tpidr_el1)?;
        for (i, value) in self.registers.iter().enumerate() {
            writeln!(f, "X{i}: 0x{value:016x}")?;
        }
        Ok(())
    }
}

fn show_vector_table_index(ix: usize) -> Option<&'static str> {
    match ix {
        0 => Some("Synchronous EL1t"),
        1 => Some("IRQ EL1t"),
        2 => Some("FIQ EL1t"),
        3 => Some("SError EL1t"),
        4 => Some("Synchronous EL1h"),
        5 => Some("IRQ EL1h"),
        6 => Some("FIQ EL1h"),
        7 => Some("SError EL1h"),
        8 => Some("Synchronous 64-bit EL0"),
        9 => Some("IRQ 64-bit EL0"),
        10 => Some("FIQ 64-bit EL0"),
        11 => Some("SError 64-bit EL0"),
        12 => Some("Synchronous 32-bit EL0"),
        13 => Some("IRQ 32-bit EL0"),
        14 => Some("FIQ 32-bit EL0"),
        15 => Some("SError 32-bit EL0"),
        _ => None,
    }
}
