//
// Copyright 2023, Colias Group, LLC
// Copyright (c) 2020 Arm Limited
//
// SPDX-License-Identifier: MIT
//

use core::ffi::c_uint;

use crate::{
    const_helpers::u32_into_usize, sel4_cfg, sel4_cfg_enum, sel4_cfg_match, sys,
    ObjectBlueprintArch, ObjectTypeArch,
};

#[sel4_cfg(KERNEL_MCS)]
use crate::const_helpers::usize_max;

#[sel4_cfg(KERNEL_MCS)]
pub const MIN_SCHED_CONTEXT_BITS: usize = u32_into_usize(sys::seL4_MinSchedContextBits);

/// Corresponds to `seL4_ObjectType`.
#[sel4_cfg_enum]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ObjectType {
    Untyped,
    Endpoint,
    Notification,
    CNode,
    TCB,
    #[sel4_cfg(KERNEL_MCS)]
    SchedContext,
    #[sel4_cfg(KERNEL_MCS)]
    Reply,
    Arch(ObjectTypeArch),
}

impl ObjectType {
    pub const fn into_sys(self) -> c_uint {
        #[sel4_cfg_match]
        match self {
            Self::Untyped => sys::api_object::seL4_UntypedObject,
            Self::Endpoint => sys::api_object::seL4_EndpointObject,
            Self::Notification => sys::api_object::seL4_NotificationObject,
            Self::CNode => sys::api_object::seL4_CapTableObject,
            Self::TCB => sys::api_object::seL4_TCBObject,
            #[sel4_cfg(KERNEL_MCS)]
            Self::SchedContext => sys::api_object::seL4_SchedContextObject,
            #[sel4_cfg(KERNEL_MCS)]
            Self::Reply => sys::api_object::seL4_ReplyObject,
            Self::Arch(arch) => arch.into_sys(),
        }
    }
}

impl From<ObjectTypeArch> for ObjectType {
    fn from(ty: ObjectTypeArch) -> Self {
        Self::Arch(ty)
    }
}

/// An object description for [`Untyped::untyped_retype`](crate::Untyped::untyped_retype).
#[sel4_cfg_enum]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectBlueprint {
    Untyped {
        size_bits: usize,
    },
    Endpoint,
    Notification,
    CNode {
        size_bits: usize,
    },
    TCB,
    #[sel4_cfg(KERNEL_MCS)]
    SchedContext {
        size_bits: usize,
    },
    #[sel4_cfg(KERNEL_MCS)]
    Reply,
    Arch(ObjectBlueprintArch),
}

impl ObjectBlueprint {
    pub const fn ty(self) -> ObjectType {
        #[sel4_cfg_match]
        match self {
            Self::Untyped { .. } => ObjectType::Untyped,
            Self::Endpoint => ObjectType::Endpoint,
            Self::Notification => ObjectType::Notification,
            Self::CNode { .. } => ObjectType::CNode,
            Self::TCB => ObjectType::TCB,
            #[sel4_cfg(KERNEL_MCS)]
            Self::SchedContext { .. } => ObjectType::SchedContext,
            #[sel4_cfg(KERNEL_MCS)]
            Self::Reply { .. } => ObjectType::Reply,
            Self::Arch(arch) => ObjectType::Arch(arch.ty()),
        }
    }

    pub const fn api_size_bits(self) -> Option<usize> {
        #[sel4_cfg_match]
        match self {
            Self::Untyped { size_bits } => Some(size_bits),
            Self::CNode { size_bits } => Some(size_bits),
            #[sel4_cfg(KERNEL_MCS)]
            Self::SchedContext { size_bits } => Some(size_bits),
            _ => None,
        }
    }

    pub const fn physical_size_bits(self) -> usize {
        #[sel4_cfg_match]
        match self {
            Self::Untyped { size_bits } => size_bits,
            Self::Endpoint => u32_into_usize(sys::seL4_EndpointBits),
            Self::Notification => u32_into_usize(sys::seL4_NotificationBits),
            Self::CNode { size_bits } => u32_into_usize(sys::seL4_SlotBits) + size_bits,
            Self::TCB => u32_into_usize(sys::seL4_TCBBits),
            #[sel4_cfg(KERNEL_MCS)]
            Self::SchedContext { size_bits } => usize_max(MIN_SCHED_CONTEXT_BITS, size_bits),
            #[sel4_cfg(KERNEL_MCS)]
            Self::Reply => u32_into_usize(sys::seL4_ReplyBits),
            Self::Arch(arch) => arch.physical_size_bits(),
        }
    }

    pub const fn asid_pool() -> Self {
        Self::Untyped {
            size_bits: u32_into_usize(sys::seL4_ASIDPoolBits),
        }
    }
}

impl From<ObjectBlueprintArch> for ObjectBlueprint {
    fn from(blueprint: ObjectBlueprintArch) -> Self {
        Self::Arch(blueprint)
    }
}
