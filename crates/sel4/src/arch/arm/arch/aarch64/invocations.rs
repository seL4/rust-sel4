use sel4_config::sel4_cfg;

use crate::{
    local_cptr::*, CapRights, Error, FrameType, IntermediateTranslationStructureType,
    InvocationContext, LocalCPtr, RelativeCPtr, Result, VCPUReg, VMAttributes, Word,
};

impl<C: InvocationContext> VCPU<C> {
    pub fn set_tcb(self, tcb: TCB) -> Result<()> {
        Error::wrap(
            self.invoke(|cptr, ipc_buffer| {
                ipc_buffer.seL4_ARM_VCPU_SetTCB(cptr.bits(), tcb.bits())
            }),
        )
    }

    pub fn read_regs(self, field: VCPUReg) -> Result<Word> {
        let res = self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.seL4_ARM_VCPU_ReadRegs(cptr.bits(), field.into_sys().try_into().unwrap())
        });
        Error::or(res.error.try_into().unwrap(), res.value)
    }

    pub fn write_regs(self, field: VCPUReg, value: Word) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.seL4_ARM_VCPU_WriteRegs(
                cptr.bits(),
                field.into_sys().try_into().unwrap(),
                value,
            )
        }))
    }

    pub fn ack_vppi(self, irq: Word) -> Result<()> {
        Error::wrap(
            self.invoke(|cptr, ipc_buffer| ipc_buffer.seL4_ARM_VCPU_AckVPPI(cptr.bits(), irq)),
        )
    }

    pub fn inject_irq(self, virq: u16, priority: u8, group: u8, index: u8) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.seL4_ARM_VCPU_InjectIRQ(cptr.bits(), virq, priority, group, index)
        }))
    }
}

impl<T: FrameType, C: InvocationContext> LocalCPtr<T, C> {
    pub fn map(self, pgd: PGD, vaddr: usize, rights: CapRights, attrs: VMAttributes) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.seL4_ARM_Page_Map(
                cptr.bits(),
                pgd.bits(),
                vaddr.try_into().unwrap(),
                rights.into_inner(),
                attrs.into_inner(),
            )
        }))
    }

    pub fn unmap(self) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| ipc_buffer.seL4_ARM_Page_Unmap(cptr.bits())))
    }

    pub fn get_address(self) -> Result<usize> {
        let ret = self.invoke(|cptr, ipc_buffer| ipc_buffer.seL4_ARM_Page_GetAddress(cptr.bits()));
        match Error::from_sys(ret.error.try_into().unwrap()) {
            None => Ok(ret.paddr.try_into().unwrap()),
            Some(err) => Err(err),
        }
    }
}

impl<T: IntermediateTranslationStructureType, C: InvocationContext> LocalCPtr<T, C> {
    pub fn map_translation_structure(
        self,
        vspace: PGD,
        vaddr: usize,
        attr: VMAttributes,
    ) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            T::_map_raw(
                ipc_buffer,
                cptr.bits(),
                vspace.bits(),
                vaddr.try_into().unwrap(),
                attr.into_inner(),
            )
        }))
    }
}

impl<C: InvocationContext> IRQControl<C> {
    // TODO structured trigger type
    #[sel4_cfg(not(MAX_NUM_NODES = "1"))]
    pub fn get_trigger_core(
        self,
        irq: Word,
        trigger: Word,
        target: Word,
        dst: &RelativeCPtr,
    ) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.seL4_IRQControl_GetTriggerCore(
                cptr.bits(),
                irq,
                trigger,
                dst.root().bits(),
                dst.path().bits(),
                dst.path().depth_for_kernel(),
                target,
            )
        }))
    }
}

impl<C: InvocationContext> IRQHandler<C> {
    pub fn ack(self) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| ipc_buffer.seL4_IRQHandler_Ack(cptr.bits())))
    }

    pub fn set_notification(self, notification: Notification) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.seL4_IRQHandler_SetNotification(cptr.bits(), notification.bits())
        }))
    }

    pub fn clear(self) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| ipc_buffer.seL4_IRQHandler_Clear(cptr.bits())))
    }
}

impl<C: InvocationContext> ASIDControl<C> {
    pub fn make_pool(self, untyped: Untyped, dst: &RelativeCPtr) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.seL4_ARM_ASIDControl_MakePool(
                cptr.bits(),
                untyped.bits(),
                dst.root().bits(),
                dst.path().bits(),
                dst.path().depth_for_kernel(),
            )
        }))
    }
}

impl<C: InvocationContext> ASIDPool<C> {
    pub fn assign(self, pd: PGD) -> Result<()> {
        Error::wrap(
            self.invoke(|cptr, ipc_buffer| {
                ipc_buffer.seL4_ARM_ASIDPool_Assign(cptr.bits(), pd.bits())
            }),
        )
    }
}
