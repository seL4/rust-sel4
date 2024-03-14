//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4::{ObjectBlueprint, VmAttributes};

use crate::{cap, Badge, Cap, FillEntryContentBootInfoId, Object, Rights};

impl<'a, D, M> Object<'a, D, M> {
    pub fn blueprint(&self) -> Option<ObjectBlueprint> {
        Some(sel4::sel4_cfg_wrap_match! {
            match self {
                Object::Untyped(obj) => ObjectBlueprint::Untyped {
                    size_bits: obj.size_bits,
                },
                Object::Endpoint => ObjectBlueprint::Endpoint,
                Object::Notification => ObjectBlueprint::Notification,
                Object::CNode(obj) => ObjectBlueprint::CNode {
                    size_bits: obj.size_bits,
                },
                Object::Tcb(_) => ObjectBlueprint::Tcb,
                #[sel4_cfg(all(ARCH_AARCH64, ARM_HYPERVISOR_SUPPORT))]
                Object::VCpu => sel4::ObjectBlueprintArch::VCpu.into(),
                Object::Frame(obj) => sel4::FrameObjectType::from_bits(obj.size_bits).unwrap().blueprint(),
                #[sel4_cfg(ARCH_AARCH64)]
                Object::PageTable(obj) => {
                    // assert!(obj.level.is_none()); // sanity check // TODO
                    if obj.is_root {
                        sel4::ObjectBlueprintSeL4Arch::VSpace.into()
                    } else {
                        sel4::ObjectBlueprintArch::PT.into()
                    }
                }
                #[sel4_cfg(ARCH_AARCH32)]
                Object::PageTable(obj) => {
                    // assert!(obj.level.is_none()); // sanity check // TODO
                    if obj.is_root {
                        sel4::ObjectBlueprintSeL4Arch::PD.into()
                    } else {
                        sel4::ObjectBlueprintArch::PT.into()
                    }
                }
                #[sel4_cfg(any(ARCH_RISCV64, ARCH_RISCV32))]
                Object::PageTable(obj) => {
                    assert!(obj.level.is_none()); // sanity check
                    sel4::ObjectBlueprintArch::PageTable.into()
                }
                #[sel4_cfg(ARCH_X86_64)]
                Object::PageTable(obj) => {
                    let level = obj.level.unwrap();
                    assert_eq!(obj.is_root, level == 0); // sanity check
                    sel4::TranslationTableObjectType::from_level(level.into()).unwrap().blueprint()
                }
                Object::AsidPool(_) => ObjectBlueprint::asid_pool(),
                #[sel4_cfg(KERNEL_MCS)]
                Object::SchedContext(obj) => ObjectBlueprint::SchedContext {
                    size_bits: obj.size_bits,
                },
                #[sel4_cfg(KERNEL_MCS)]
                Object::Reply => ObjectBlueprint::Reply,
                _ => return None,
            }
        })
    }
}

impl Cap {
    pub fn rights(&self) -> Option<&Rights> {
        Some(match self {
            Cap::Endpoint(cap) => &cap.rights,
            Cap::Notification(cap) => &cap.rights,
            Cap::Frame(cap) => &cap.rights,
            _ => return None,
        })
    }

    pub fn badge(&self) -> Option<Badge> {
        Some(match self {
            Cap::Endpoint(cap) => cap.badge,
            Cap::Notification(cap) => cap.badge,
            Cap::CNode(cap) => {
                sel4::CNodeCapData::new(cap.guard, cap.guard_size.try_into().unwrap()).into_word()
            }
            _ => return None,
        })
    }
}

impl From<&Rights> for sel4::CapRights {
    fn from(rights: &Rights) -> Self {
        Self::new(rights.grant_reply, rights.grant, rights.read, rights.write)
    }
}

impl From<&FillEntryContentBootInfoId> for sel4::BootInfoExtraId {
    fn from(id: &FillEntryContentBootInfoId) -> Self {
        match id {
            FillEntryContentBootInfoId::Fdt => sel4::BootInfoExtraId::Fdt,
        }
    }
}

pub trait HasVmAttributes {
    fn vm_attributes(&self) -> VmAttributes;
}

impl HasVmAttributes for cap::Frame {
    fn vm_attributes(&self) -> VmAttributes {
        vm_attributes_from_whether_cached(self.cached)
    }
}

impl HasVmAttributes for cap::PageTable {
    fn vm_attributes(&self) -> VmAttributes {
        default_vm_attributes_for_page_table()
    }
}

sel4::sel4_cfg_if! {
    if #[sel4_cfg(any(ARCH_AARCH64, ARCH_AARCH32))] {
        const CACHED: VmAttributes = VmAttributes::PAGE_CACHEABLE;
        const UNCACHED: VmAttributes = VmAttributes::DEFAULT;
    } else if #[sel4_cfg(any(ARCH_RISCV64, ARCH_RISCV32))] {
        const CACHED: VmAttributes = VmAttributes::DEFAULT;
        const UNCACHED: VmAttributes = VmAttributes::NONE;
    } else if #[sel4_cfg(ARCH_X86_64)] {
        const CACHED: VmAttributes = VmAttributes::DEFAULT;
        const UNCACHED: VmAttributes = VmAttributes::CACHE_DISABLED;
    }
}

pub fn vm_attributes_from_whether_cached(cached: bool) -> VmAttributes {
    if cached {
        CACHED
    } else {
        UNCACHED
    }
}

fn default_vm_attributes_for_page_table() -> VmAttributes {
    VmAttributes::default()
}
