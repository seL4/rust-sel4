//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use core::marker::PhantomData;
use core::ops::Range;

use sel4_config::sel4_cfg;

use crate::{
    cap_type,
    const_helpers::{u32_into_usize, usize_into_word, word_into_usize},
    sys, CPtr, CPtrBits, CapType, LocalCPtr,
};

/// The index of a slot in the initial thread's root CNode.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Slot<T: CapType = cap_type::Unspecified> {
    index: usize,
    _phantom: PhantomData<T>,
}

impl<T: CapType> Slot<T> {
    const fn from_sys(slot: u32) -> Self {
        Self::from_index(u32_into_usize(slot))
    }

    pub const fn from_index(index: usize) -> Self {
        Self {
            index,
            _phantom: PhantomData,
        }
    }

    pub const fn index(&self) -> usize {
        self.index
    }

    pub const fn cptr_bits(&self) -> CPtrBits {
        usize_into_word(self.index)
    }

    pub const fn cptr(&self) -> CPtr {
        CPtr::from_bits(self.cptr_bits())
    }

    pub const fn local_cptr(&self) -> LocalCPtr<T> {
        self.cptr().cast()
    }

    pub const fn cast<T1: CapType>(&self) -> Slot<T1> {
        Slot::from_index(self.index)
    }

    pub const fn upcast(&self) -> Slot {
        self.cast()
    }
}

impl Slot {
    pub const fn downcast<T: CapType>(&self) -> Slot<T> {
        self.cast()
    }
}

/// Corresponds to `seL4_SlotRegion`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SlotRegion<T: CapType> {
    range: Range<usize>,
    _phantom: PhantomData<T>,
}

impl<T: CapType> SlotRegion<T> {
    pub(crate) const fn from_range(range: Range<usize>) -> Self {
        Self {
            range,
            _phantom: PhantomData,
        }
    }

    pub(crate) const fn from_sys(sys: sys::seL4_SlotRegion) -> Self {
        Self::from_range(word_into_usize(sys.start)..word_into_usize(sys.end))
    }

    pub const fn start(&self) -> usize {
        self.range.start
    }

    pub const fn end(&self) -> usize {
        self.range.end
    }

    pub const fn range(&self) -> Range<usize> {
        self.start()..self.end()
    }

    pub fn len(&self) -> usize {
        self.range.len()
    }

    pub fn index(&self, i: usize) -> Slot<T> {
        assert!(i < self.len());
        Slot::from_index(self.range.start + i)
    }
}

pub mod slot {
    use super::{cap_type, sel4_cfg, sys, Slot};

    macro_rules! mk {
        [
            $(
                $(#[$outer:meta])*
                ($name:ident, $cap_type:ident, $sys_name:ident),
            )*
        ] => {
            $(
                $(#[$outer])*
                pub const $name: Slot<cap_type::$cap_type> = Slot::from_sys(sys::seL4_RootCNodeCapSlots::$sys_name);
            )*
        };
    }

    mk![
        (NULL, Null, seL4_CapNull),
        (TCB, TCB, seL4_CapInitThreadTCB),
        (CNODE, CNode, seL4_CapInitThreadCNode),
        (VSPACE, VSpace, seL4_CapInitThreadVSpace),
        (IRQ_CONTROL, IRQControl, seL4_CapIRQControl),
        (ASID_CONTROL, ASIDControl, seL4_CapASIDControl),
        (ASID_POOL, ASIDPool, seL4_CapInitThreadASIDPool),
        #[cfg(any())] // TODO
        (IO_PORT_CONTROL, Null, seL4_CapIOPortControl),
        #[cfg(any())] // TODO
        (IO_SPACE, Null, seL4_CapIOSpace),
        (BOOT_INFO_FRAME, Granule, seL4_CapBootInfoFrame),
        (IPC_BUFFER, Granule, seL4_CapInitThreadIPCBuffer),
        #[cfg(any())] // TODO
        (DOMAIN, Null, seL4_CapDomain),
        #[cfg(any())] // TODO
        (SMMU_SID_CONTROL, Null, seL4_CapSMMUSIDControl),
        #[cfg(any())] // TODO
        (SMMU_CB_CONTROL, Null, seL4_CapSMMUCBControl),
        #[sel4_cfg(KERNEL_MCS)]
        (SC, SchedControl, seL4_CapInitThreadSC),
        #[cfg(any())] // TODO
        (SMC, Null, seL4_CapSMC),
    ];
}

// NOTE(rustc_wishlist) use ! once #![never_type] is stabilized
#[cfg(feature = "state")]
pub fn suspend_self<T>() -> T {
    slot::TCB.local_cptr().tcb_suspend().unwrap();

    unreachable!()
}
