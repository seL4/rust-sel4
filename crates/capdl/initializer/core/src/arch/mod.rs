use sel4::sel4_cfg;

#[sel4_cfg(any(ARCH_AARCH32, ARCH_AARCH64))]
mod imp {
    pub(crate) type FrameType1 = sel4::cap_type::LargePage;
    pub(crate) type FrameType2 = sel4::cap_type::HugePage;

    pub(crate) fn init_user_context(
        regs: &mut sel4::UserContext,
        extra: &capdl_types::object::TCBExtraInfo,
    ) {
        *regs.pc_mut() = extra.ip;
        *regs.sp_mut() = extra.sp;
        *regs.spsr_mut() = extra.spsr;
        for (i, value) in extra.gprs.iter().enumerate() {
            *regs.gpr_mut(i.try_into().unwrap()) = *value;
        }
    }
}

#[sel4_cfg(any(ARCH_IA32, ARCH_X86_64))]
#[path = "x86/mod.rs"]
mod imp {
    pub(crate) type FrameType1 = sel4::cap_type::LargePage;
    pub(crate) type FrameType2 = sel4::cap_type::HugePage;

    pub(crate) fn init_user_context(
        regs: &mut sel4::UserContext,
        extra: &capdl_types::object::TCBExtraInfo,
    ) {
        *regs.pc_mut() = extra.ip;
        *regs.sp_mut() = extra.sp;
        for (i, value) in extra.gprs.iter().enumerate() {
            *regs.gpr_mut(i.try_into().unwrap()) = *value;
        }
    }
}

#[sel4_cfg(any(ARCH_RISCV32, ARCH_RISCV64))]
#[path = "riscv/mod.rs"]
mod imp {}

pub(crate) use imp::*;

pub(crate) mod frame_types {
    use sel4::FrameType;

    pub(crate) use super::{FrameType1, FrameType2};

    pub(crate) type FrameType0 = sel4::cap_type::Granule;

    pub(crate) const FRAME_SIZE_0_BITS: usize = FrameType0::FRAME_SIZE.bits();
    pub(crate) const FRAME_SIZE_1_BITS: usize = FrameType1::FRAME_SIZE.bits();
}

sel4::sel4_cfg_if! {
    if #[cfg(ARCH_AARCH64)] {
        const CACHED: sel4::VMAttributes = sel4::VMAttributes::PAGE_CACHEABLE;
        const UNCACHED: sel4::VMAttributes = sel4::VMAttributes::DEFAULT;
    } else if #[cfg(ARCH_X86_64)] {
        const CACHED: sel4::VMAttributes = sel4::VMAttributes::DEFAULT;
        const UNCACHED: sel4::VMAttributes = sel4::VMAttributes::CACHE_DISABLED;
    }
}

pub(crate) fn vm_attributes_from_whether_cached(cached: bool) -> sel4::VMAttributes {
    if cached {
        CACHED
    } else {
        UNCACHED
    }
}
