//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::arch::asm;

fn cache_line_size() -> usize {
    let ctr: u64;
    unsafe {
        asm!("mrs {}, ctr_el0", out(reg) ctr, options(nomem, nostack));
    }
    let dminline = (ctr >> 16) & 0xf;
    let words_per_cache_line = 1 << dminline;
    4 * words_per_cache_line
}

pub(crate) unsafe fn clean_dcache_range(start: usize, size: usize) {
    let line_size = cache_line_size();
    let end = start.saturating_add(size);
    let mut line_addr = start & !(line_size - 1);

    while line_addr < end {
        unsafe {
            asm!("dc cvac, {}", in(reg) line_addr, options(nostack));
        }
        line_addr += line_size;
    }

    unsafe {
        asm!("dsb sy", options(nostack));
    }
}

pub(crate) unsafe fn invalidate_icache_all() {
    unsafe {
        asm! {
            r#"
                ic iallu
                dsb sy
                isb
            "#,
            options(nostack),
        };
    }
}
