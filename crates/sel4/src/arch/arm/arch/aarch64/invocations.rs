use sel4_config::sel4_cfg;

use crate::{
    local_cptr::*, CapRights, Error, FrameType, InvocationContext, LocalCPtr, RelativeCPtr, Result,
    TranslationTableType, VCPUReg, VMAttributes, Word,
};

#[sel4_cfg(ARM_HYPERVISOR_SUPPORT)]
impl<C: InvocationContext> VCPU<C> {
    pub fn vcpu_set_tcb(self, tcb: TCB) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer
                .inner_mut()
                .seL4_ARM_VCPU_SetTCB(cptr.bits(), tcb.bits())
        }))
    }

    pub fn vcpu_read_regs(self, field: VCPUReg) -> Result<Word> {
        let res = self.invoke(|cptr, ipc_buffer| {
            ipc_buffer
                .inner_mut()
                .seL4_ARM_VCPU_ReadRegs(cptr.bits(), field.into_sys().try_into().unwrap())
        });
        Error::or(res.error.try_into().unwrap(), res.value)
    }

    pub fn vcpu_write_regs(self, field: VCPUReg, value: Word) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_ARM_VCPU_WriteRegs(
                cptr.bits(),
                field.into_sys().try_into().unwrap(),
                value,
            )
        }))
    }

    pub fn vcpu_ack_vppi(self, irq: Word) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer
                .inner_mut()
                .seL4_ARM_VCPU_AckVPPI(cptr.bits(), irq)
        }))
    }

    pub fn vcpu_inject_irq(self, virq: u16, priority: u8, group: u8, index: u8) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_ARM_VCPU_InjectIRQ(
                cptr.bits(),
                virq,
                priority,
                group,
                index,
            )
        }))
    }
}

impl<T: FrameType, C: InvocationContext> LocalCPtr<T, C> {
    pub fn frame_map(
        self,
        pgd: PGD,
        vaddr: usize,
        rights: CapRights,
        attrs: VMAttributes,
    ) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_ARM_Page_Map(
                cptr.bits(),
                pgd.bits(),
                vaddr.try_into().unwrap(),
                rights.into_inner(),
                attrs.into_inner(),
            )
        }))
    }

    pub fn frame_unmap(self) -> Result<()> {
        Error::wrap(
            self.invoke(|cptr, ipc_buffer| ipc_buffer.inner_mut().seL4_ARM_Page_Unmap(cptr.bits())),
        )
    }

    pub fn frame_get_address(self) -> Result<usize> {
        let ret = self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_ARM_Page_GetAddress(cptr.bits())
        });
        match Error::from_sys(ret.error.try_into().unwrap()) {
            None => Ok(ret.paddr.try_into().unwrap()),
            Some(err) => Err(err),
        }
    }
}

impl<T: TranslationTableType, C: InvocationContext> LocalCPtr<T, C> {
    pub fn translation_table_map(
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
    pub fn irq_control_get_trigger_core(
        self,
        irq: Word,
        trigger: Word,
        target: Word,
        dst: &RelativeCPtr,
    ) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_IRQControl_GetTriggerCore(
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

impl<C: InvocationContext> ASIDControl<C> {
    pub fn asid_control_make_pool(self, untyped: Untyped, dst: &RelativeCPtr) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_ARM_ASIDControl_MakePool(
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
    pub fn asid_pool_assign(self, pd: PGD) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer
                .inner_mut()
                .seL4_ARM_ASIDPool_Assign(cptr.bits(), pd.bits())
        }))
    }
}
