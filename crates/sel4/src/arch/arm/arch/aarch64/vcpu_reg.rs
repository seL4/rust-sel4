use sel4_config::sel4_cfg_enum;

use crate::sys;

#[repr(u32)]
#[allow(non_camel_case_types)]
#[sel4_cfg_enum]
pub enum VCPUReg {
    SCTLR = sys::seL4_VCPUReg::seL4_VCPUReg_SCTLR,
    TTBR0 = sys::seL4_VCPUReg::seL4_VCPUReg_TTBR0,
    TTBR1 = sys::seL4_VCPUReg::seL4_VCPUReg_TTBR1,
    TCR = sys::seL4_VCPUReg::seL4_VCPUReg_TCR,
    MAIR = sys::seL4_VCPUReg::seL4_VCPUReg_MAIR,
    AMAIR = sys::seL4_VCPUReg::seL4_VCPUReg_AMAIR,
    CIDR = sys::seL4_VCPUReg::seL4_VCPUReg_CIDR,
    ACTLR = sys::seL4_VCPUReg::seL4_VCPUReg_ACTLR,
    CPACR = sys::seL4_VCPUReg::seL4_VCPUReg_CPACR,
    AFSR0 = sys::seL4_VCPUReg::seL4_VCPUReg_AFSR0,
    AFSR1 = sys::seL4_VCPUReg::seL4_VCPUReg_AFSR1,
    ESR = sys::seL4_VCPUReg::seL4_VCPUReg_ESR,
    FAR = sys::seL4_VCPUReg::seL4_VCPUReg_FAR,
    ISR = sys::seL4_VCPUReg::seL4_VCPUReg_ISR,
    VBAR = sys::seL4_VCPUReg::seL4_VCPUReg_VBAR,
    TPIDR_EL1 = sys::seL4_VCPUReg::seL4_VCPUReg_TPIDR_EL1,
    #[sel4_cfg(not(MAX_NUM_NODES = "1"))]
    VMPIDR_EL2 = sys::seL4_VCPUReg::seL4_VCPUReg_VMPIDR_EL2,
    SP_EL1 = sys::seL4_VCPUReg::seL4_VCPUReg_SP_EL1,
    ELR_EL1 = sys::seL4_VCPUReg::seL4_VCPUReg_ELR_EL1,
    SPSR_EL1 = sys::seL4_VCPUReg::seL4_VCPUReg_SPSR_EL1,
    CNTV_CTL = sys::seL4_VCPUReg::seL4_VCPUReg_CNTV_CTL,
    CNTV_CVAL = sys::seL4_VCPUReg::seL4_VCPUReg_CNTV_CVAL,
    CNTVOFF = sys::seL4_VCPUReg::seL4_VCPUReg_CNTVOFF,
}

impl VCPUReg {
    pub const fn into_sys(self) -> sys::seL4_VCPUReg::Type {
        self as sys::seL4_VCPUReg::Type
    }
}
