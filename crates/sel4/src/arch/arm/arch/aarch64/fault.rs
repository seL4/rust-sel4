use sel4_config::sel4_cfg;

use crate::{
    fault::{
        CapFault, UnknownSyscall, UserException, VCPUFault, VGICMaintenance, VMFault, VPPIEvent,
    },
    Word,
};

impl CapFault {
    // TODO
}

impl UnknownSyscall {
    pub fn fault_ip(&self) -> Word {
        self.inner().get_FaultIP()
    }

    pub fn sp(&self) -> Word {
        self.inner().get_SP()
    }

    pub fn lr(&self) -> Word {
        self.inner().get_LR()
    }

    pub fn spsr(&self) -> Word {
        self.inner().get_SPSR()
    }

    pub fn syscall(&self) -> Word {
        self.inner().get_Syscall()
    }

    pub fn gpr(&self, ix: usize) -> u64 {
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

impl UserException {
    // TODO
}

impl VMFault {
    pub fn ip(&self) -> Word {
        self.inner().get_IP()
    }

    pub fn addr(&self) -> Word {
        self.inner().get_Addr()
    }

    pub fn is_prefetch(&self) -> bool {
        self.inner().get_PrefetchFault() != 0
    }

    pub fn fsr(&self) -> Word {
        self.inner().get_FSR()
    }
}

#[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
impl VGICMaintenance {
    pub fn idx(&self) -> Option<Word> {
        match self.inner().get_IDX() {
            Word::MAX => None,
            idx => Some(idx),
        }
    }
}

#[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
impl VCPUFault {
    pub fn hsr(&self) -> Word {
        self.inner().get_HSR()
    }
}

#[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
impl VPPIEvent {
    pub fn irq(&self) -> Word {
        self.inner().get_irq()
    }
}
