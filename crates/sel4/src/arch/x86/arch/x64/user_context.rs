use crate::{newtype_methods, sys, Word};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UserContext(sys::seL4_UserContext);

impl UserContext {
    newtype_methods!(sys::seL4_UserContext);

    pub fn pc(&self) -> &Word {
        &self.0.rip
    }

    pub fn pc_mut(&mut self) -> &mut Word {
        &mut self.0.rip
    }

    pub fn sp(&self) -> &Word {
        &self.0.rsp
    }

    pub fn sp_mut(&mut self) -> &mut Word {
        &mut self.0.rsp
    }

    pub fn gpr(&self, ix: Word) -> &Word {
        // TODO
        match ix {
            0 => &self.inner().rax,
            1 => &self.inner().rbx,
            2 => &self.inner().rcx,
            3 => &self.inner().rdx,
            4 => &self.inner().rsi,
            5 => &self.inner().rdi,
            6 => &self.inner().rbp,
            _ => panic!(),
        }
    }

    pub fn gpr_mut(&mut self, ix: Word) -> &mut Word {
        match ix {
            0 => &mut self.inner_mut().rax,
            1 => &mut self.inner_mut().rbx,
            2 => &mut self.inner_mut().rcx,
            3 => &mut self.inner_mut().rdx,
            4 => &mut self.inner_mut().rsi,
            5 => &mut self.inner_mut().rdi,
            6 => &mut self.inner_mut().rbp,
            _ => panic!(),
        }
    }
}
