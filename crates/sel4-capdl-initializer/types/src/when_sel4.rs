//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use rkyv::Archive;

use sel4::{ObjectBlueprint, VmAttributes};

use crate::{
    ArchivedCap, ArchivedFillEntryContentBootInfoId, ArchivedObject, ArchivedRights, ArchivedWord,
    cap,
};

impl<D: Archive> ArchivedObject<D> {
    pub fn blueprint(&self) -> Option<ObjectBlueprint> {
        Some(sel4::sel4_cfg_wrap_match! {
            match self {
                ArchivedObject::Untyped(obj) => ObjectBlueprint::Untyped {
                    size_bits: obj.size_bits.into(),
                },
                ArchivedObject::Endpoint => ObjectBlueprint::Endpoint,
                ArchivedObject::Notification => ObjectBlueprint::Notification,
                ArchivedObject::CNode(obj) => ObjectBlueprint::CNode {
                    size_bits: obj.size_bits.into(),
                },
                ArchivedObject::Tcb(_) => ObjectBlueprint::Tcb,
                #[sel4_cfg(any(all(ARCH_AARCH64, ARM_HYPERVISOR_SUPPORT), all(ARCH_X86_64, VTX)))]
                ArchivedObject::VCpu => sel4::ObjectBlueprintArch::VCpu.into(),
                ArchivedObject::Frame(obj) => sel4::FrameObjectType::from_bits(obj.size_bits.into()).unwrap().blueprint(),
                #[sel4_cfg(ARCH_AARCH64)]
                ArchivedObject::PageTable(obj) => {
                    // assert!(obj.level.is_none()); // sanity check // TODO
                    if obj.is_root {
                        sel4::ObjectBlueprintSeL4Arch::VSpace.into()
                    } else {
                        sel4::ObjectBlueprintArch::PT.into()
                    }
                }
                #[sel4_cfg(ARCH_AARCH32)]
                ArchivedObject::PageTable(obj) => {
                    // assert!(obj.level.is_none()); // sanity check // TODO
                    if obj.is_root {
                        sel4::ObjectBlueprintSeL4Arch::PD.into()
                    } else {
                        sel4::ObjectBlueprintArch::PT.into()
                    }
                }
                #[sel4_cfg(any(ARCH_RISCV64, ARCH_RISCV32))]
                ArchivedObject::PageTable(_obj) => {
                    // assert!(obj.level.is_none()); // sanity check // TODO
                    sel4::ObjectBlueprintArch::PageTable.into()
                }
                #[sel4_cfg(ARCH_X86_64)]
                ArchivedObject::PageTable(obj) => {
                    let level = obj.level.unwrap();
                    assert_eq!(obj.is_root, level == 0); // sanity check
                    sel4::TranslationTableObjectType::from_level(level.into()).unwrap().blueprint()
                }
                ArchivedObject::AsidPool(_) => ObjectBlueprint::asid_pool(),
                #[sel4_cfg(KERNEL_MCS)]
                ArchivedObject::SchedContext(obj) => ObjectBlueprint::SchedContext {
                    size_bits: obj.size_bits.into(),
                },
                #[sel4_cfg(KERNEL_MCS)]
                ArchivedObject::Reply => ObjectBlueprint::Reply,
                _ => return None,
            }
        })
    }
}

impl ArchivedCap {
    pub fn rights(&self) -> Option<sel4::CapRights> {
        Some(
            match self {
                ArchivedCap::Endpoint(cap) => &cap.rights,
                ArchivedCap::Notification(cap) => &cap.rights,
                ArchivedCap::Frame(cap) => &cap.rights,
                _ => return None,
            }
            .to_sel4(),
        )
    }

    pub fn badge(&self) -> Option<sel4::Badge> {
        Some(match self {
            ArchivedCap::Endpoint(cap) => cap.badge.to_sel4(),
            ArchivedCap::Notification(cap) => cap.badge.to_sel4(),
            ArchivedCap::CNode(cap) => {
                sel4::CNodeCapData::new(cap.guard.to_sel4(), cap.guard_size.into()).into_word()
            }

            _ => return None,
        })
    }
}

impl ArchivedWord {
    #[allow(clippy::useless_conversion)]
    pub fn to_sel4(&self) -> sel4::Word {
        self.0.to_native().try_into().unwrap()
    }
}

impl ArchivedRights {
    pub fn to_sel4(&self) -> sel4::CapRights {
        sel4::CapRights::new(self.grant_reply, self.grant, self.read, self.write)
    }
}

impl ArchivedFillEntryContentBootInfoId {
    pub fn to_sel4(&self) -> sel4::BootInfoExtraId {
        match self {
            Self::Fdt => sel4::BootInfoExtraId::Fdt,
        }
    }
}

pub trait HasVmAttributes {
    fn vm_attributes(&self) -> VmAttributes;
}

impl HasVmAttributes for cap::ArchivedFrame {
    fn vm_attributes(&self) -> VmAttributes {
        vm_attributes_from_whether_cached_and_exec(self.cached, self.executable)
    }
}

impl HasVmAttributes for cap::ArchivedPageTable {
    fn vm_attributes(&self) -> VmAttributes {
        default_vm_attributes_for_page_table()
    }
}

sel4::sel4_cfg_if! {
    if #[sel4_cfg(any(ARCH_AARCH64, ARCH_AARCH32))] {
        const CACHED: VmAttributes = VmAttributes::DEFAULT;
        const UNCACHED: VmAttributes = VmAttributes::NONE;
        const NO_EXEC: VmAttributes = VmAttributes::EXECUTE_NEVER;
    } else if #[sel4_cfg(any(ARCH_RISCV64, ARCH_RISCV32))] {
        const CACHED: VmAttributes = VmAttributes::DEFAULT;
        const UNCACHED: VmAttributes = VmAttributes::NONE;
        const NO_EXEC: VmAttributes = VmAttributes::EXECUTE_NEVER;
    } else if #[sel4_cfg(ARCH_X86_64)] {
        const CACHED: VmAttributes = VmAttributes::DEFAULT;
        const UNCACHED: VmAttributes = VmAttributes::CACHE_DISABLED;
    }
}

// Allow these because on some architectures, certain variables are not touched.
#[allow(unused_variables, unused_assignments)]
pub fn vm_attributes_from_whether_cached_and_exec(cached: bool, executable: bool) -> VmAttributes {
    let mut vmattr = VmAttributes::NONE;
    if cached {
        vmattr = CACHED;
    } else {
        vmattr = UNCACHED;
    }

    sel4::sel4_cfg_if! {
        if #[sel4_cfg(any(ARCH_AARCH64, ARCH_AARCH32, ARCH_RISCV64, ARCH_RISCV32))] {
            if !executable {
                vmattr |= NO_EXEC
            }
        }
    }

    vmattr
}

fn default_vm_attributes_for_page_table() -> VmAttributes {
    VmAttributes::default()
}
