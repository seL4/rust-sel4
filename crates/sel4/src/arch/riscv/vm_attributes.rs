//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

use crate::{newtype_methods, sys};

/// Corresponds to `seL4_RISCV_VMAttributes`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct VMAttributes(sys::seL4_RISCV_VMAttributes::Type);

impl VMAttributes {
    pub const NONE: Self = Self::from_inner(0);
    pub const DEFAULT: Self =
        Self::from_inner(sys::seL4_RISCV_VMAttributes::seL4_RISCV_Default_VMAttributes);
    pub const EXECUTE_NEVER: Self =
        Self::from_inner(sys::seL4_RISCV_VMAttributes::seL4_RISCV_ExecuteNever);

    newtype_methods!(sys::seL4_RISCV_VMAttributes::Type);

    pub const fn has(self, rhs: Self) -> bool {
        self.into_inner() & rhs.into_inner() != 0
    }
}

impl Default for VMAttributes {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Not for VMAttributes {
    type Output = Self;
    fn not(self) -> Self {
        Self::from_inner(self.into_inner().not())
    }
}

impl BitOr for VMAttributes {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self::from_inner(self.into_inner().bitor(rhs.into_inner()))
    }
}

impl BitOrAssign for VMAttributes {
    fn bitor_assign(&mut self, rhs: Self) {
        self.inner_mut().bitor_assign(rhs.into_inner());
    }
}

impl BitAnd for VMAttributes {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self::from_inner(self.into_inner().bitand(rhs.into_inner()))
    }
}

impl BitAndAssign for VMAttributes {
    fn bitand_assign(&mut self, rhs: Self) {
        self.inner_mut().bitand_assign(rhs.into_inner());
    }
}
