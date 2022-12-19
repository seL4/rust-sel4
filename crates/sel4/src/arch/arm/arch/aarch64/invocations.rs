use sel4_config::sel4_cfg;

use crate::{
    local_cptr::*, CapRights, Error, FrameType, IntermediateTranslationStructureType, LocalCPtr,
    RelativeCPtr, Result, VCPUReg, VMAttributes, Word, IPC_BUFFER,
};

impl VCPU {
    pub fn set_tcb(&self, tcb: TCB) -> Result<()> {
        Error::wrap(
            IPC_BUFFER
                .borrow_mut()
                .as_mut()
                .unwrap()
                .seL4_ARM_VCPU_SetTCB(self.bits(), tcb.bits()),
        )
    }

    pub fn read_regs(&self, field: VCPUReg) -> Result<Word> {
        let res = IPC_BUFFER
            .borrow_mut()
            .as_mut()
            .unwrap()
            .seL4_ARM_VCPU_ReadRegs(self.bits(), field.into_sys().try_into().unwrap());
        Error::or(res.error.try_into().unwrap(), res.value)
    }

    pub fn write_regs(&self, field: VCPUReg, value: Word) -> Result<()> {
        Error::wrap(
            IPC_BUFFER
                .borrow_mut()
                .as_mut()
                .unwrap()
                .seL4_ARM_VCPU_WriteRegs(self.bits(), field.into_sys().try_into().unwrap(), value),
        )
    }

    pub fn ack_vppi(&self, irq: Word) -> Result<()> {
        Error::wrap(
            IPC_BUFFER
                .borrow_mut()
                .as_mut()
                .unwrap()
                .seL4_ARM_VCPU_AckVPPI(self.bits(), irq),
        )
    }

    pub fn inject_irq(&self, virq: u16, priority: u8, group: u8, index: u8) -> Result<()> {
        Error::wrap(
            IPC_BUFFER
                .borrow_mut()
                .as_mut()
                .unwrap()
                .seL4_ARM_VCPU_InjectIRQ(self.bits(), virq, priority, group, index),
        )
    }
}

impl<T: FrameType> LocalCPtr<T> {
    pub fn map(
        &self,
        pgd: PGD,
        vaddr: usize,
        rights: CapRights,
        attrs: VMAttributes,
    ) -> Result<()> {
        Error::wrap(IPC_BUFFER.borrow_mut().as_mut().unwrap().seL4_ARM_Page_Map(
            self.bits(),
            pgd.bits(),
            vaddr.try_into().unwrap(),
            rights.into_inner(),
            attrs.into_inner(),
        ))
    }

    pub fn unmap(&self) -> Result<()> {
        Error::wrap(
            IPC_BUFFER
                .borrow_mut()
                .as_mut()
                .unwrap()
                .seL4_ARM_Page_Unmap(self.bits()),
        )
    }

    pub fn get_address(&self) -> Result<usize> {
        let ret = IPC_BUFFER
            .borrow_mut()
            .as_mut()
            .unwrap()
            .seL4_ARM_Page_GetAddress(self.bits());
        match Error::from_sys(ret.error.try_into().unwrap()) {
            None => Ok(ret.paddr.try_into().unwrap()),
            Some(err) => Err(err),
        }
    }
}

impl<T: IntermediateTranslationStructureType> LocalCPtr<T> {
    pub fn map_translation_structure(
        &self,
        vspace: PGD,
        vaddr: usize,
        attr: VMAttributes,
    ) -> Result<()> {
        Error::wrap(T::_map_raw(
            self.bits(),
            vspace.bits(),
            vaddr.try_into().unwrap(),
            attr.into_inner(),
        ))
    }
}

impl IRQControl {
    // TODO structured trigger type
    #[sel4_cfg(not(MAX_NUM_NODES = "1"))]
    pub fn get_trigger_core(
        &self,
        irq: Word,
        trigger: Word,
        target: Word,
        dst: &RelativeCPtr,
    ) -> Result<()> {
        Error::wrap(
            IPC_BUFFER
                .borrow_mut()
                .as_mut()
                .unwrap()
                .seL4_IRQControl_GetTriggerCore(
                    self.bits(),
                    irq,
                    trigger,
                    dst.root().bits(),
                    dst.path().bits(),
                    dst.path().depth_for_kernel(),
                    target,
                ),
        )
    }
}

impl IRQHandler {
    pub fn ack(&self) -> Result<()> {
        Error::wrap(
            IPC_BUFFER
                .borrow_mut()
                .as_mut()
                .unwrap()
                .seL4_IRQHandler_Ack(self.bits()),
        )
    }

    pub fn set_notification(&self, notification: Notification) -> Result<()> {
        Error::wrap(
            IPC_BUFFER
                .borrow_mut()
                .as_mut()
                .unwrap()
                .seL4_IRQHandler_SetNotification(self.bits(), notification.bits()),
        )
    }

    pub fn clear(&self) -> Result<()> {
        Error::wrap(
            IPC_BUFFER
                .borrow_mut()
                .as_mut()
                .unwrap()
                .seL4_IRQHandler_Clear(self.bits()),
        )
    }
}

impl ASIDControl {
    pub fn make_pool(&self, untyped: Untyped, dst: &RelativeCPtr) -> Result<()> {
        Error::wrap(
            IPC_BUFFER
                .borrow_mut()
                .as_mut()
                .unwrap()
                .seL4_ARM_ASIDControl_MakePool(
                    self.bits(),
                    untyped.bits(),
                    dst.root().bits(),
                    dst.path().bits(),
                    dst.path().depth_for_kernel(),
                ),
        )
    }
}

impl ASIDPool {
    pub fn assign(&self, pd: PGD) -> Result<()> {
        Error::wrap(
            IPC_BUFFER
                .borrow_mut()
                .as_mut()
                .unwrap()
                .seL4_ARM_ASIDPool_Assign(self.bits(), pd.bits()),
        )
    }
}
