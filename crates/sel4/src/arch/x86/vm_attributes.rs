//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

use crate::{newtype_methods, sel4_cfg_if, sys};

/// Corresponds to `seL4_X86_VMAttributes`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct VmAttributes(sys::seL4_X86_VMAttributes::Type);

impl VmAttributes {
    pub const NONE: Self = Self::from_inner(0);
    pub const DEFAULT: Self =
        Self::from_inner(sys::seL4_X86_VMAttributes::seL4_X86_Default_VMAttributes);
    pub const CACHE_DISABLED: Self =
        Self::from_inner(sys::seL4_X86_VMAttributes::seL4_X86_CacheDisabled);

    sel4_cfg_if! {
        if #[sel4_cfg(all(ARCH_X86_64, VTX))] {
            pub const EPT_DEFAULT: Self =
                Self::from_inner(sys::seL4_X86_EPT_VMAttributes::seL4_X86_EPT_Default_VMAttributes);
            pub const EPT_CACHE_DISABLED: Self =
                Self::from_inner(sys::seL4_X86_EPT_VMAttributes::seL4_X86_EPT_Uncacheable);
        }
    }

    newtype_methods!(pub sys::seL4_X86_VMAttributes::Type);

    pub const fn has(self, rhs: Self) -> bool {
        self.into_inner() & rhs.into_inner() != 0
    }
}

impl Default for VmAttributes {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Not for VmAttributes {
    type Output = Self;
    fn not(self) -> Self {
        Self::from_inner(self.into_inner().not())
    }
}

impl BitOr for VmAttributes {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self::from_inner(self.into_inner().bitor(rhs.into_inner()))
    }
}

impl BitOrAssign for VmAttributes {
    fn bitor_assign(&mut self, rhs: Self) {
        self.inner_mut().bitor_assign(rhs.into_inner());
    }
}

impl BitAnd for VmAttributes {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self::from_inner(self.into_inner().bitand(rhs.into_inner()))
    }
}

impl BitAndAssign for VmAttributes {
    fn bitand_assign(&mut self, rhs: Self) {
        self.inner_mut().bitand_assign(rhs.into_inner());
    }
}
