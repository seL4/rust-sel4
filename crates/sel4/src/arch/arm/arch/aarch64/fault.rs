use sel4_config::{sel4_cfg};

use crate::{Word, CapFault, UnknownSyscall, UserException, VMFault, VCPUFault, VGICMaintenance, VPPIEvent};

impl CapFault {
    // TODO
}

impl UnknownSyscall {
    pub fn fault_ip(&self) -> Word {
        self.0.get_FaultIP()
    }

    pub fn sp(&self) -> Word {
        self.0.get_SP()
    }

    pub fn lr(&self) -> Word {
        self.0.get_LR()
    }

    pub fn spsr(&self) -> Word {
        self.0.get_SPSR()
    }

    pub fn syscall(&self) -> Word {
        self.0.get_Syscall()
    }

    pub fn gpr(&self, ix: usize) -> u64 {
        match ix {
            0 => self.0.get_X0(),
            1 => self.0.get_X1(),
            2 => self.0.get_X2(),
            3 => self.0.get_X3(),
            4 => self.0.get_X4(),
            5 => self.0.get_X5(),
            6 => self.0.get_X6(),
            7 => self.0.get_X7(),
            _ => panic!(),
        }
    }
}

impl UserException {
    // TODO
}

impl VMFault {
    pub fn ip(&self) -> Word {
        self.0.get_IP()
    }

    pub fn addr(&self) -> Word {
        self.0.get_Addr()
    }

    pub fn is_prefetch(&self) -> bool {
        self.0.get_PrefetchFault() != 0
    }

    pub fn fsr(&self) -> Word {
        self.0.get_FSR()
    }
}

#[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
impl VGICMaintenance {
    pub fn idx(&self) -> Option<Word> {
        match self.0.get_IDX() {
            Word::MAX => None,
            idx => Some(idx),
        }
    }
}

#[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
impl VCPUFault {
    pub fn hsr(&self) -> Word {
        self.0.get_HSR()
    }
}

#[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
impl VPPIEvent {
    pub fn irq(&self) -> Word {
        self.0.get_irq()
    }
}
