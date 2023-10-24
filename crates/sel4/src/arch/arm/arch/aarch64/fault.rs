//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use crate::{fault::UnknownSyscall, Word};

impl UnknownSyscall {
    pub fn spsr(&self) -> Word {
        self.inner().get_SPSR()
    }

    pub fn gpr(&self, ix: usize) -> Word {
        match ix {
            0 => self.inner().get_X0(),
            1 => self.inner().get_X1(),
            2 => self.inner().get_X2(),
            3 => self.inner().get_X3(),
            4 => self.inner().get_X4(),
            5 => self.inner().get_X5(),
            6 => self.inner().get_X6(),
            7 => self.inner().get_X7(),
            _ => panic!(),
        }
    }
}
