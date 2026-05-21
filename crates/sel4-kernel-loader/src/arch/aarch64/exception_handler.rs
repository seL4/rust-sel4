//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::arch::global_asm;
use core::fmt;

use aarch64_cpu::registers::{ESR_EL2, FAR_EL2, Readable, TPIDR_EL1};

use crate::arch::{Arch, ArchImpl};
use crate::fmt::debug_println;

#[used]
static mut EXCEPTION_REGISTER_STATE: Registers = [0; _];

unsafe extern "C" fn exception_handler(vector_table_index: u64) -> ! {
    let exception = Exception {
        vector_table_index,
        esr: ESR_EL2.get(),
        far: FAR_EL2.get(),
        tpidr_el1: TPIDR_EL1.get(),
        registers: unsafe { EXCEPTION_REGISTER_STATE },
    };
    debug_println!("!!! Exception:\n{}", exception);
    ArchImpl::idle()
}

const NUM_REGISTERS: usize = 32;

type Registers = [u64; NUM_REGISTERS];

struct Exception {
    vector_table_index: u64,
    esr: u64,
    far: u64,
    tpidr_el1: u64,
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

fn show_vector_table_index(ix: u64) -> Option<&'static str> {
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

global_asm! {
    r#"
        .macro ventry id
        .p2align 7
            // Juggle registers using stack to introduce struct pointer into context
            stp     x2, x3, [sp, #-16]
            ldr     x2, ={exception_register_state}
            stp     x0, x1, [x2]
            mov     x0, x2
            ldp     x2, x3, [sp, #-16]
            stp     x2, x3, [x0, #16 * 1]
            stp     x4, x5, [x0, #16 * 2]
            stp     x6, x7, [x0, #16 * 3]
            stp     x8, x9, [x0, #16 * 4]
            stp     x10, x11, [x0, #16 * 5]
            stp     x12, x13, [x0, #16 * 6]
            stp     x14, x15, [x0, #16 * 7]
            stp     x16, x17, [x0, #16 * 8]
            stp     x18, x19, [x0, #16 * 9]
            stp     x20, x21, [x0, #16 * 10]
            stp     x22, x23, [x0, #16 * 11]
            stp     x24, x25, [x0, #16 * 12]
            stp     x26, x27, [x0, #16 * 13]
            stp     x28, x29, [x0, #16 * 14]
            mov     x0, \id
            b       {exception_handler}
        .endm

        .text

        .global arm_vector_table
        .size arm_vector_table, {size}
        .p2align 12
        arm_vector_table:
            ventry  #0
            ventry  #1
            ventry  #2
            ventry  #3
            ventry  #4
            ventry  #5
            ventry  #6
            ventry  #7
            ventry  #8
            ventry  #9
            ventry  #10
            ventry  #11
            ventry  #12
            ventry  #13
            ventry  #14
            ventry  #15
    "#,
    exception_register_state = sym EXCEPTION_REGISTER_STATE,
    exception_handler = sym exception_handler,
    size = const { 16 * (1 << 7) },
}
