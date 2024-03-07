//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use crate::{newtype_methods, sys, Word};

/// Corresponds to `seL4_UserContext`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UserContext(sys::seL4_UserContext);

impl UserContext {
    newtype_methods!(pub sys::seL4_UserContext);

    pub fn pc(&self) -> &Word {
        &self.0.pc
    }

    pub fn pc_mut(&mut self) -> &mut Word {
        &mut self.0.pc
    }

    pub fn sp(&self) -> &Word {
        &self.0.sp
    }

    pub fn sp_mut(&mut self) -> &mut Word {
        &mut self.0.sp
    }

    pub fn gpr_a(&self, ix: usize) -> &Word {
        match ix {
            0 => &self.inner().a0,
            1 => &self.inner().a1,
            2 => &self.inner().a2,
            3 => &self.inner().a3,
            4 => &self.inner().a4,
            5 => &self.inner().a5,
            6 => &self.inner().a6,
            7 => &self.inner().a7,
            _ => panic!(),
        }
    }

    pub fn gpr_a_mut(&mut self, ix: usize) -> &mut Word {
        match ix {
            0 => &mut self.inner_mut().a0,
            1 => &mut self.inner_mut().a1,
            2 => &mut self.inner_mut().a2,
            3 => &mut self.inner_mut().a3,
            4 => &mut self.inner_mut().a4,
            5 => &mut self.inner_mut().a5,
            6 => &mut self.inner_mut().a6,
            7 => &mut self.inner_mut().a7,
            _ => panic!(),
        }
    }

    pub fn c_param(&self, ix: usize) -> &Word {
        self.gpr_a(ix)
    }

    pub fn c_param_mut(&mut self, ix: usize) -> &mut Word {
        self.gpr_a_mut(ix)
    }
}
