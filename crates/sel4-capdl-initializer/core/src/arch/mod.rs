//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4::{sel4_cfg, VmAttributes};

#[sel4_cfg(any(ARCH_AARCH32, ARCH_AARCH64))]
mod imp {
    pub(crate) type FrameType1 = sel4::cap_type::LargePage;
    pub(crate) type FrameType2 = sel4::cap_type::HugePage;

    pub(crate) type PageTableType = sel4::cap_type::PT;

    pub(crate) const VSPACE_LEVELS: usize = if sel4::sel4_cfg_bool!(ARCH_AARCH64) {
        4
    } else {
        2
    };

    pub(crate) fn map_page_table(
        vspace: sel4::VSpace,
        _level: usize,
        vaddr: usize,
        cap: sel4::Unspecified,
        vm_attributes: sel4::VmAttributes,
    ) -> sel4::Result<()> {
        cap.downcast::<PageTableType>()
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

#[sel4_cfg(any(ARCH_RISCV32, ARCH_RISCV64))]
mod imp {
    pub(crate) type FrameType1 = sel4::cap_type::MegaPage;
    pub(crate) type FrameType2 = sel4::cap_type::GigaPage;

    pub(crate) type PageTableType = sel4::cap_type::PageTable;

    pub(crate) const VSPACE_LEVELS: usize = sel4::sel4_cfg_usize!(PT_LEVELS);

    pub(crate) fn map_page_table(
        vspace: sel4::VSpace,
        _level: usize,
        vaddr: usize,
        cap: sel4::Unspecified,
        vm_attributes: sel4::VmAttributes,
    ) -> sel4::Result<()> {
        cap.downcast::<PageTableType>()
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
    pub(crate) type FrameType1 = sel4::cap_type::LargePage;
    pub(crate) type FrameType2 = sel4::cap_type::HugePage;

    pub(crate) const VSPACE_LEVELS: usize = if sel4::sel4_cfg_bool!(ARCH_X86_64) {
        4
    } else {
        2
    };

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

pub(crate) mod frame_types {
    use sel4::SizedFrameType;

    pub(crate) use super::{FrameType1, FrameType2};

    pub(crate) type FrameType0 = sel4::cap_type::Granule;

    pub(crate) const FRAME_SIZE_0_BITS: usize = FrameType0::FRAME_SIZE.bits();
    pub(crate) const FRAME_SIZE_1_BITS: usize = FrameType1::FRAME_SIZE.bits();
}

sel4::sel4_cfg_if! {
    if #[sel4_cfg(ARCH_AARCH64)] {
        const CACHED: VmAttributes = VmAttributes::PAGE_CACHEABLE;
        const UNCACHED: VmAttributes = VmAttributes::DEFAULT;
    } else if #[sel4_cfg(ARCH_RISCV64)] {
        const CACHED: VmAttributes = VmAttributes::DEFAULT;
        const UNCACHED: VmAttributes = VmAttributes::NONE;
    } else if #[sel4_cfg(ARCH_X86_64)] {
        const CACHED: VmAttributes = VmAttributes::DEFAULT;
        const UNCACHED: VmAttributes = VmAttributes::CACHE_DISABLED;
    }
}

pub(crate) fn vm_attributes_from_whether_cached(cached: bool) -> VmAttributes {
    if cached {
        CACHED
    } else {
        UNCACHED
    }
}
