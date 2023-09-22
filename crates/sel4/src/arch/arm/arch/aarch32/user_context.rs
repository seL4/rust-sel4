use crate::{newtype_methods, sys, Word};

/// Corresponds to `seL4_UserContext`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UserContext(sys::seL4_UserContext);

impl UserContext {
    newtype_methods!(sys::seL4_UserContext);

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

    pub fn cpsr(&self) -> &Word {
        &self.0.cpsr
    }

    pub fn cpsr_mut(&mut self) -> &mut Word {
        &mut self.0.cpsr
    }

    pub fn gpr(&self, ix: Word) -> &Word {
        match ix {
            0 => &self.inner().r0,
            1 => &self.inner().r1,
            2 => &self.inner().r2,
            3 => &self.inner().r3,
            4 => &self.inner().r4,
            5 => &self.inner().r5,
            6 => &self.inner().r6,
            7 => &self.inner().r7,
            8 => &self.inner().r8,
            9 => &self.inner().r9,
            10 => &self.inner().r10,
            11 => &self.inner().r11,
            12 => &self.inner().r12,
            14 => &self.inner().r14,
            _ => panic!(),
        }
    }

    pub fn gpr_mut(&mut self, ix: Word) -> &mut Word {
        match ix {
            0 => &mut self.inner_mut().r0,
            1 => &mut self.inner_mut().r1,
            2 => &mut self.inner_mut().r2,
            3 => &mut self.inner_mut().r3,
            4 => &mut self.inner_mut().r4,
            5 => &mut self.inner_mut().r5,
            6 => &mut self.inner_mut().r6,
            7 => &mut self.inner_mut().r7,
            8 => &mut self.inner_mut().r8,
            9 => &mut self.inner_mut().r9,
            10 => &mut self.inner_mut().r10,
            11 => &mut self.inner_mut().r11,
            12 => &mut self.inner_mut().r12,
            14 => &mut self.inner_mut().r14,
            _ => panic!(),
        }
    }
}
