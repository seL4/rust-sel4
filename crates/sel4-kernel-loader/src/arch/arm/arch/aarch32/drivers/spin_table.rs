//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::arch::{asm, global_asm};
use core::ptr;

use aarch32_cpu::asm::sev;
use aarch32_cpu::cache::clean_data_cache_line_to_poc;

#[used]
#[unsafe(no_mangle)]
static mut spin_table_secondary_stack_bottom: usize = 0;

pub(crate) fn start_core(spin_table: &[usize], core_id: usize, sp: usize) {
    unsafe {
        ptr::addr_of_mut!(spin_table_secondary_stack_bottom).write(sp);

        let start = spin_table_secondary_entry as *const SpinTableSecondaryEntryFn as usize;
        let start_ptr = spin_table[core_id] as *mut usize;

        start_ptr.write_volatile(start);

        clean_data_cache_line_to_poc(ptr::addr_of!(spin_table_secondary_stack_bottom) as u32);

        // Barrier ensure both strl and dc cvac happen before sev
        asm!("dsb sy");
        sev();
    }
}

type SpinTableSecondaryEntryFn = extern "C" fn() -> !;

unsafe extern "C" {
    fn spin_table_secondary_entry() -> !;
}

global_asm! {
    r#"
        .extern spin_table_secondary_stack_bottom
        .extern secondary_entry

        .section .text

        .global spin_table_secondary_entry
        spin_table_secondary_entry:
            ldr r0, =spin_table_secondary_stack_bottom
            ldr r0, [r0]
            b secondary_entry
    "#
}
