//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4::{sel4_cfg, SizedFrameType};

#[sel4_cfg(any(ARCH_AARCH64, ARCH_AARCH32))]
mod imp {
    use sel4::{cap_type, sel4_cfg};

    pub(crate) const NUM_LEVELS: usize = if sel4::sel4_cfg_bool!(ARCH_AARCH64) {
        4
    } else {
        2
    };

    #[sel4_cfg(ARCH_AARCH64)]
    pub(crate) fn level_bits(_level: usize) -> usize {
        cap_type::PT::INDEX_BITS
    }

    #[sel4_cfg(ARCH_AARCH32)]
    pub(crate) fn level_bits(level: usize) -> usize {
        match level {
            0 => cap_type::PD::INDEX_BITS,
            1 => cap_type::PT::INDEX_BITS,
            _ => unreachable!(),
        }
    }

    pub(crate) fn map_page_table(
        vspace: sel4::VSpace,
        _level: usize,
        vaddr: usize,
        cap: sel4::Unspecified,
        vm_attributes: sel4::VmAttributes,
    ) -> sel4::Result<()> {
        cap.downcast::<sel4::cap_type::PT>()
            .pt_map(vspace, vaddr, vm_attributes)
    }

    pub(crate) fn init_user_context(
        regs: &mut sel4::UserContext,
        extra: &sel4_capdl_initializer_types::object::TcbExtraInfo,
    ) {
        *regs.pc_mut() = extra.ip;
        *regs.sp_mut() = extra.sp;
        for (i, value) in extra.gprs.iter().enumerate() {
            *regs.gpr_mut(i) = *value;
        }
    }
}

#[sel4_cfg(any(ARCH_RISCV64, ARCH_RISCV32))]
mod imp {
    use sel4::cap_type;

    pub(crate) const NUM_LEVELS: usize = sel4::sel4_cfg_usize!(PT_LEVELS);

    pub(crate) fn level_bits(_level: usize) -> usize {
        cap_type::PageTable::INDEX_BITS
    }

    pub(crate) fn map_page_table(
        vspace: sel4::VSpace,
        _level: usize,
        vaddr: usize,
        cap: sel4::Unspecified,
        vm_attributes: sel4::VmAttributes,
    ) -> sel4::Result<()> {
        cap.downcast::<sel4::cap_type::PageTable>()
            .page_table_map(vspace, vaddr, vm_attributes)
    }

    pub(crate) fn init_user_context(
        regs: &mut sel4::UserContext,
        extra: &sel4_capdl_initializer_types::object::TcbExtraInfo,
    ) {
        *regs.pc_mut() = extra.ip;
        *regs.sp_mut() = extra.sp;
        for (i, value) in extra.gprs.iter().enumerate() {
            *regs.gpr_a_mut(i) = *value;
        }
    }
}

#[sel4_cfg(ARCH_X86_64)]
mod imp {
    pub(crate) const NUM_LEVELS: usize = 4;

    pub(crate) fn level_bits(_level: usize) -> usize {
        9
    }

    pub(crate) fn map_page_table(
        vspace: sel4::VSpace,
        level: usize,
        vaddr: usize,
        cap: sel4::Unspecified,
        vm_attributes: sel4::VmAttributes,
    ) -> sel4::Result<()> {
        match level {
            1 => cap
                .downcast::<sel4::cap_type::PDPT>()
                .pdpt_map(vspace, vaddr, vm_attributes),
            2 => cap
                .downcast::<sel4::cap_type::PageDirectory>()
                .page_directory_map(vspace, vaddr, vm_attributes),
            3 => cap.downcast::<sel4::cap_type::PageTable>().page_table_map(
                vspace,
                vaddr,
                vm_attributes,
            ),
            _ => panic!(),
        }
    }

    pub(crate) fn init_user_context(
        regs: &mut sel4::UserContext,
        extra: &sel4_capdl_initializer_types::object::TcbExtraInfo,
    ) {
        *regs.pc_mut() = extra.ip;
        *regs.sp_mut() = extra.sp;
        for (i, value) in extra.gprs.iter().enumerate() {
            *regs.gpr_mut(i) = *value;
        }
    }
}

pub(crate) use imp::*;

pub(crate) fn step_bits(level: usize) -> usize {
    ((level + 1)..NUM_LEVELS).map(level_bits).sum::<usize>()
        + sel4::cap_type::Granule::FRAME_SIZE.bits()
}

pub(crate) fn span_bits(level: usize) -> usize {
    step_bits(level) + level_bits(level)
}
