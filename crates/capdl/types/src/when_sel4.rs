use sel4::{ObjectBlueprint, ObjectBlueprintAArch64, ObjectBlueprintArm, VMAttributes};

use crate::{cap, FillEntryContentBootInfoId, Object, Rights};

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
            Object::SmallPage(_) => ObjectBlueprintArm::SmallPage.into(),
            Object::LargePage(_) => ObjectBlueprintArm::LargePage.into(),
            Object::PT(_) => ObjectBlueprintArm::PT.into(),
            Object::PD(_) => ObjectBlueprintArm::PD.into(),
            Object::PUD(_) => ObjectBlueprintAArch64::PUD.into(),
            Object::PGD(_) => ObjectBlueprintAArch64::PGD.into(),
            Object::ASIDPool(_) => ObjectBlueprint::asid_pool(),
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

impl HasVMAttributes for cap::SmallPage {
    fn vm_attributes(&self) -> VMAttributes {
        vm_attributes_from_whether_cached(self.cached)
    }
}

impl HasVMAttributes for cap::LargePage {
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
