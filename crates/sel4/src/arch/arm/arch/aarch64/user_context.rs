//
// Copyright 2023, Colias Group, LLC
// Copyright (c) 2020 Arm Limited
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

    pub fn spsr(&self) -> &Word {
        &self.0.spsr
    }

    pub fn spsr_mut(&mut self) -> &mut Word {
        &mut self.0.spsr
    }

    pub fn gpr(&self, ix: usize) -> &Word {
        match ix {
            0 => &self.inner().x0,
            1 => &self.inner().x1,
            2 => &self.inner().x2,
            3 => &self.inner().x3,
            4 => &self.inner().x4,
            5 => &self.inner().x5,
            6 => &self.inner().x6,
            7 => &self.inner().x7,
            8 => &self.inner().x8,
            9 => &self.inner().x9,
            10 => &self.inner().x10,
            11 => &self.inner().x11,
            12 => &self.inner().x12,
            13 => &self.inner().x13,
            14 => &self.inner().x14,
            15 => &self.inner().x15,
            16 => &self.inner().x16,
            17 => &self.inner().x17,
            18 => &self.inner().x18,
            19 => &self.inner().x19,
            20 => &self.inner().x20,
            21 => &self.inner().x21,
            22 => &self.inner().x22,
            23 => &self.inner().x23,
            24 => &self.inner().x24,
            25 => &self.inner().x25,
            26 => &self.inner().x26,
            27 => &self.inner().x27,
            28 => &self.inner().x28,
            29 => &self.inner().x29,
            30 => &self.inner().x30,
            _ => panic!(),
        }
    }

    pub fn gpr_mut(&mut self, ix: usize) -> &mut Word {
        match ix {
            0 => &mut self.inner_mut().x0,
            1 => &mut self.inner_mut().x1,
            2 => &mut self.inner_mut().x2,
            3 => &mut self.inner_mut().x3,
            4 => &mut self.inner_mut().x4,
            5 => &mut self.inner_mut().x5,
            6 => &mut self.inner_mut().x6,
            7 => &mut self.inner_mut().x7,
            8 => &mut self.inner_mut().x8,
            9 => &mut self.inner_mut().x9,
            10 => &mut self.inner_mut().x10,
            11 => &mut self.inner_mut().x11,
            12 => &mut self.inner_mut().x12,
            13 => &mut self.inner_mut().x13,
            14 => &mut self.inner_mut().x14,
            15 => &mut self.inner_mut().x15,
            16 => &mut self.inner_mut().x16,
            17 => &mut self.inner_mut().x17,
            18 => &mut self.inner_mut().x18,
            19 => &mut self.inner_mut().x19,
            20 => &mut self.inner_mut().x20,
            21 => &mut self.inner_mut().x21,
            22 => &mut self.inner_mut().x22,
            23 => &mut self.inner_mut().x23,
            24 => &mut self.inner_mut().x24,
            25 => &mut self.inner_mut().x25,
            26 => &mut self.inner_mut().x26,
            27 => &mut self.inner_mut().x27,
            28 => &mut self.inner_mut().x28,
            29 => &mut self.inner_mut().x29,
            30 => &mut self.inner_mut().x30,
            _ => panic!(),
        }
    }
}
