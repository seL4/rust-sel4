use core::ffi::c_uint;

use crate::{sys, ObjectBlueprintArch, ObjectTypeArch};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ObjectType {
    Untyped,
    Endpoint,
    Notification,
    CNode,
    TCB,
    Arch(ObjectTypeArch),
}

impl ObjectType {
    pub const fn into_sys(self) -> c_uint {
        match self {
            Self::Untyped => sys::api_object::seL4_UntypedObject,
            Self::Endpoint => sys::api_object::seL4_EndpointObject,
            Self::Notification => sys::api_object::seL4_NotificationObject,
            Self::CNode => sys::api_object::seL4_CapTableObject,
            Self::TCB => sys::api_object::seL4_TCBObject,
            Self::Arch(arch) => arch.into_sys(),
        }
    }
}

impl From<ObjectTypeArch> for ObjectType {
    fn from(ty: ObjectTypeArch) -> Self {
        Self::Arch(ty)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectBlueprint {
    Untyped { size_bits: usize },
    Endpoint,
    Notification,
    CNode { size_bits: usize },
    TCB,
    Arch(ObjectBlueprintArch),
}

impl ObjectBlueprint {
    pub fn ty(self) -> ObjectType {
        match self {
            Self::Untyped { .. } => ObjectType::Untyped,
            Self::Endpoint => ObjectType::Endpoint,
            Self::Notification => ObjectType::Notification,
            Self::CNode { .. } => ObjectType::CNode,
            Self::TCB => ObjectType::TCB,
            Self::Arch(arch) => arch.ty(),
        }
    }

    pub const fn api_size_bits(self) -> Option<usize> {
        match self {
            Self::Untyped { size_bits } => Some(size_bits),
            Self::CNode { size_bits } => Some(size_bits),
            _ => None,
        }
    }

    pub fn physical_size_bits(self) -> usize {
        match self {
            Self::Untyped { size_bits } => size_bits,
            Self::Endpoint => sys::seL4_EndpointBits.try_into().unwrap(),
            Self::Notification => sys::seL4_NotificationBits.try_into().unwrap(),
            Self::CNode { size_bits } => usize::try_from(sys::seL4_SlotBits).unwrap() + size_bits,
            Self::TCB => sys::seL4_TCBBits.try_into().unwrap(),
            Self::Arch(arch) => arch.physical_size_bits(),
        }
    }

    pub fn asid_pool() -> Self {
        Self::Untyped {
            size_bits: sys::seL4_ASIDPoolBits.try_into().unwrap(),
        }
    }
}

impl From<ObjectBlueprintArch> for ObjectBlueprint {
    fn from(blueprint: ObjectBlueprintArch) -> Self {
        Self::Arch(blueprint)
    }
}
