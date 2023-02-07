use sel4::{ObjectBlueprint, ObjectBlueprintAArch64, ObjectBlueprintArm, VMAttributes};

use crate::{cap, Badge, Cap, FillEntryContentBootInfoId, Object, Rights};

impl<'a, F> Object<'a, F> {
    pub fn blueprint(&self) -> Option<ObjectBlueprint> {
        Some(match self {
            Object::Untyped(obj) => ObjectBlueprint::Untyped {
                size_bits: obj.size_bits,
            },
            Object::Endpoint => ObjectBlueprint::Endpoint,
            Object::Notification => ObjectBlueprint::Notification,
            Object::CNode(obj) => ObjectBlueprint::CNode {
                size_bits: obj.size_bits,
            },
            Object::TCB(_) => ObjectBlueprint::TCB,
            Object::VCPU => ObjectBlueprintArm::VCPU.into(),
            Object::Frame(obj) => match obj.size_bits {
                sel4::FrameSize::SMALL_BITS => ObjectBlueprintArm::SmallPage.into(),
                sel4::FrameSize::LARGE_BITS => ObjectBlueprintArm::LargePage.into(),
                _ => panic!(),
            },
            Object::PT(_) => ObjectBlueprintArm::PT.into(),
            Object::PD(_) => ObjectBlueprintArm::PD.into(),
            Object::PUD(_) => ObjectBlueprintAArch64::PUD.into(),
            Object::PGD(_) => ObjectBlueprintAArch64::PGD.into(),
            Object::ASIDPool(_) => ObjectBlueprint::asid_pool(),
            _ => return None,
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

pub trait HasVMAttributes {
    fn vm_attributes(&self) -> VMAttributes;
}

impl HasVMAttributes for cap::Frame {
    fn vm_attributes(&self) -> VMAttributes {
        vm_attributes_from_whether_cached(self.cached)
    }
}

impl HasVMAttributes for cap::PGD {
    fn vm_attributes(&self) -> VMAttributes {
        default_vm_attributes_for_translation_structure()
    }
}

impl HasVMAttributes for cap::PUD {
    fn vm_attributes(&self) -> VMAttributes {
        default_vm_attributes_for_translation_structure()
    }
}

impl HasVMAttributes for cap::PD {
    fn vm_attributes(&self) -> VMAttributes {
        default_vm_attributes_for_translation_structure()
    }
}

impl HasVMAttributes for cap::PT {
    fn vm_attributes(&self) -> VMAttributes {
        default_vm_attributes_for_translation_structure()
    }
}

fn vm_attributes_from_whether_cached(cached: bool) -> VMAttributes {
    VMAttributes::default()
        & !(if !cached {
            VMAttributes::PAGE_CACHEABLE
        } else {
            VMAttributes::NONE
        })
}

fn default_vm_attributes_for_translation_structure() -> VMAttributes {
    VMAttributes::default()
}
