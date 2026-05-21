//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::arch::naked_asm;

use riscv::register::satp;

use sel4_config::{sel4_cfg_if, sel4_cfg_usize};

use crate::{
    arch::Arch,
    main, secondary_main,
    this_image::{kernel_boot_level_0_table, stacks::PRIMARY_STACK_BOTTOM},
};

macro_rules! asm_prolog {
    () => {
        r#"
            .extern __global_pointer$
            .option push
            .option norelax
            1:  auipc gp, %pcrel_hi(__global_pointer$)
                addi  gp, gp, %pcrel_lo(1b)
            .option pop
        "#
    };
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.start")]
extern "C" fn _start(hart_id: usize, dtb: usize) -> ! {
    naked_asm! {
        cfg_select! {
            target_arch = "riscv64" => r#"
                .macro lx dst, src
                    ld \dst, \src
                .endm
            "#,
            target_arch = "riscv32" => r#"
                .macro lx dst, src
                    lw \dst, \src
                .endm
            "#,
        }
        asm_prolog!(),
        r#"
                la sp, {stack_bottom}
                lx sp, (sp)
                la s0, {arch_main}
                jr s0
            .purgem lx
        "#,
        stack_bottom = sym PRIMARY_STACK_BOTTOM,
        arch_main = sym arch_main,
    }
}

#[unsafe(naked)]
pub(crate) extern "C" fn start_secondary(hart_id: usize) -> ! {
    naked_asm! {
        asm_prolog!(),
        r#"
            mv sp, a1
            la s0, {arch_secondary_main}
            jr s0
        "#,
        arch_secondary_main = sym arch_secondary_main,
    }
}

extern "C" fn arch_main(hart_id: usize, dtb: usize) -> ! {
    main(hart_id, dtb)
}

extern "C" fn arch_secondary_main(hart_id: usize) -> ! {
    secondary_main(hart_id)
}

pub(crate) enum ArchImpl {}

impl Arch for ArchImpl {
    fn physical_to_logical_core_id(physical_core_id: usize) -> Option<usize> {
        let logical_core_id = physical_core_id.checked_sub(sel4_cfg_usize!(FIRST_HART_ID))?;
        if logical_core_id < sel4_cfg_usize!(MAX_NUM_NODES) {
            Some(logical_core_id)
        } else {
            None
        }
    }

    fn logical_to_physical_core_id(logical_core_id: usize) -> usize {
        logical_core_id + sel4_cfg_usize!(FIRST_HART_ID)
    }

    fn idle() -> ! {
        loop {
            riscv::asm::wfi();
        }
    }

    fn prepare_to_enter_kernel(_core_id: usize) {
        switch_page_tables();
    }
}

fn switch_page_tables() {
    const MODE: satp::Mode = sel4_cfg_if! {
        if #[sel4_cfg(all(SEL4_ARCH = "riscv64", PT_LEVELS = "3"))] {
            satp::Mode::Sv39
        } else if #[sel4_cfg(all(SEL4_ARCH = "riscv32", PT_LEVELS = "2"))] {
            satp::Mode::Sv32
        } else {
            compiler_error!("unsupported configuration");
        }
    };

    let ppn = kernel_boot_level_0_table.get() >> 12;
    riscv::asm::sfence_vma_all();

    unsafe {
        satp::set(MODE, 0, ppn);
    }

    riscv::asm::fence_i();
}
