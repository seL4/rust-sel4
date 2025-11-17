//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use rkyv::Archive;

use crate::{
    ArchivedCapSlot, ArchivedCapTableEntry, CapSlot, CapTableEntry, IsArchivedCap, IsCap, cap,
    object,
};

pub trait HasCapTable {
    fn slots(&self) -> &[CapTableEntry];

    fn slot_as<T: IsCap>(&self, slot: CapSlot) -> Option<&T> {
        self.slots().as_ref().iter().find_map(|entry| {
            if entry.slot == slot {
                Some(entry.cap.as_().unwrap())
            } else {
                None
            }
        })
    }
}

pub trait HasArchivedCapTable {
    fn slots(&self) -> &[ArchivedCapTableEntry];

    fn slot_as<T: IsArchivedCap>(&self, slot: ArchivedCapSlot) -> Option<&T> {
        self.slots().as_ref().iter().find_map(|entry| {
            if entry.slot == slot {
                Some(entry.cap.as_().unwrap())
            } else {
                None
            }
        })
    }
}

macro_rules! alias_cap_table {
    ($obj_ty:ty | $archived_obj_ty:ty {
        $(
            $accessor_name:ident: $cap_ty:ident = $slot_name:ident($n:expr) $(@$optional:ident)?
        ),* $(,)?
    }) => {
        impl $obj_ty {
            $(
                pub const $slot_name: CapSlot = CapSlot($n);

                alias_cap_table_helper! {
                    $accessor_name: &cap::$cap_ty = $slot_name $(@$optional)?
                }
            )*
        }

        impl $archived_obj_ty {
            $(
                pub const $slot_name: ArchivedCapSlot = ArchivedCapSlot(<u32 as Archive>::Archived::from_native($n));

                alias_cap_table_helper! {
                    $accessor_name: &<cap::$cap_ty as Archive>::Archived = $slot_name $(@$optional)?
                }
            )*
        }
    };
}

macro_rules! alias_cap_table_helper {
    ($accessor_name:ident: $ty:ty = $slot_name:ident) => {
        pub fn $accessor_name(&self) -> $ty {
            self.slot_as(Self::$slot_name).unwrap()
        }
    };
    ($accessor_name:ident: $ty:ty = $slot_name:ident @optional) => {
        pub fn $accessor_name(&self) -> Option<$ty> {
            self.slot_as(Self::$slot_name)
        }
    };
}

// NOTE
// Magic constants must be kept in sync with capDL-tool.

alias_cap_table! {
    object::Tcb | object::ArchivedTcb {
        cspace: CNode = SLOT_CSPACE(0),
        vspace: PageTable = SLOT_VSPACE(1),
        ipc_buffer: Frame = SLOT_IPC_BUFFER(4),
        mcs_fault_ep: Endpoint = SLOT_FAULT_EP(5) @optional,
        sc: SchedContext = SLOT_SC(6) @optional,
        temp_fault_ep: Endpoint = SLOT_TEMP_FAULT_EP(7) @optional,
        bound_notification: Notification = SLOT_BOUND_NOTIFICATION(8) @optional,
        vcpu: VCpu = SLOT_VCPU(9) @optional,
        x86_eptpml4: PageTable = SLOT_X86_EPTPML4(10) @optional,
    }
}

alias_cap_table! {
    object::Irq | object::ArchivedIrq {
        notification: Notification = SLOT_NOTIFICATION(0) @optional,
    }
}

alias_cap_table! {
    object::ArmIrq | object::ArchivedArmIrq {
        notification: Notification = SLOT_NOTIFICATION(0) @optional,
    }
}

alias_cap_table! {
    object::IrqMsi | object::ArchivedIrqMsi {
        notification: Notification = SLOT_NOTIFICATION(0) @optional,
    }
}

alias_cap_table! {
    object::IrqIOApic | object::ArchivedIrqIOApic {
        notification: Notification = SLOT_NOTIFICATION(0) @optional,
    }
}

alias_cap_table! {
    object::RiscvIrq | object::ArchivedRiscvIrq {
        notification: Notification = SLOT_NOTIFICATION(0) @optional,
    }
}
