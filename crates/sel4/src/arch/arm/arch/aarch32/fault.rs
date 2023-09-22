use crate::{fault::UnknownSyscall, Word};

impl UnknownSyscall {
    pub fn cpsr(&self) -> Word {
        self.inner().get_CPSR()
    }

    pub fn gpr(&self, ix: usize) -> Word {
        match ix {
            0 => self.inner().get_R0(),
            1 => self.inner().get_R1(),
            2 => self.inner().get_R2(),
            3 => self.inner().get_R3(),
            4 => self.inner().get_R4(),
            5 => self.inner().get_R5(),
            6 => self.inner().get_R6(),
            7 => self.inner().get_R7(),
            _ => panic!(),
        }
    }
}
