use core::mem;

use sel4_config::sel4_cfg;

use crate::{
    local_cptr::*, CNodeCapData, CPtr, CapRights, Error, InvocationContext, ObjectBlueprint,
    RelativeCPtr, Result, UserContext, Word,
};

// NOTE
// &self enables convenient use of Deref at the cost of indirection. Is this appropriate?

impl<C: InvocationContext> Untyped<C> {
    pub fn retype(
        self,
        blueprint: &ObjectBlueprint,
        dst: &RelativeCPtr,
        dst_offset: usize,
        num_objects: usize,
    ) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_Untyped_Retype(
                cptr.bits(),
                blueprint.ty().into_sys().try_into().unwrap(),
                blueprint.api_size_bits().unwrap_or(0).try_into().unwrap(),
                dst.root().bits(),
                dst.path().bits(),
                dst.path().depth().try_into().unwrap(),
                dst_offset.try_into().unwrap(),
                num_objects.try_into().unwrap(),
            )
        }))
    }
}

const USER_CONTEXT_MAX_REG_COUNT: usize = mem::size_of::<UserContext>() / mem::size_of::<Word>();

impl<C: InvocationContext> TCB<C> {
    pub fn read_registers(self, suspend: bool, count: Word) -> Result<UserContext> {
        let mut regs: UserContext = Default::default();
        let err = self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_TCB_ReadRegisters(
                cptr.bits(),
                suspend.into(),
                0,
                count,
                regs.inner_mut(),
            )
        });
        Error::or(err, regs)
    }

    pub fn read_all_registers(self, suspend: bool) -> Result<UserContext> {
        self.read_registers(suspend, USER_CONTEXT_MAX_REG_COUNT.try_into().unwrap())
    }

    // HACK should not be mut
    pub fn write_registers(self, resume: bool, count: Word, regs: &mut UserContext) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_TCB_WriteRegisters(
                cptr.bits(),
                resume.into(),
                0,
                count,
                regs.inner_mut(),
            )
        }))
    }

    pub fn write_all_registers(self, resume: bool, regs: &mut UserContext) -> Result<()> {
        self.write_registers(resume, USER_CONTEXT_MAX_REG_COUNT.try_into().unwrap(), regs)
    }

    pub fn resume(self) -> Result<()> {
        Error::wrap(
            self.invoke(|cptr, ipc_buffer| ipc_buffer.inner_mut().seL4_TCB_Resume(cptr.bits())),
        )
    }

    pub fn suspend(self) -> Result<()> {
        Error::wrap(
            self.invoke(|cptr, ipc_buffer| ipc_buffer.inner_mut().seL4_TCB_Suspend(cptr.bits())),
        )
    }

    pub fn configure(
        self,
        fault_ep: CPtr,
        cspace_root: CNode,
        cspace_root_data: CNodeCapData,
        vspace_root: PGD,
        ipc_buffer: Word,
        ipc_buffer_frame: SmallPage,
    ) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ctx_ipc_buffer| {
            ctx_ipc_buffer.inner_mut().seL4_TCB_Configure(
                cptr.bits(),
                fault_ep.bits(),
                cspace_root.bits(),
                cspace_root_data.into_word(),
                vspace_root.bits(),
                0, /* HACK */
                ipc_buffer,
                ipc_buffer_frame.bits(),
            )
        }))
    }

    pub fn set_sched_params(self, authority: TCB, mcp: Word, priority: Word) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_TCB_SetSchedParams(
                cptr.bits(),
                authority.bits(),
                mcp,
                priority,
            )
        }))
    }

    #[sel4_cfg(not(MAX_NUM_NODES = "1"))]
    pub fn set_affinity(self, affinity: Word) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer
                .inner_mut()
                .seL4_TCB_SetAffinity(cptr.bits(), affinity)
        }))
    }

    pub fn set_tls_base(self, tls_base: Word) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer
                .inner_mut()
                .seL4_TCB_SetTLSBase(cptr.bits(), tls_base)
        }))
    }

    pub fn bind_notification(self, notification: Notification) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer
                .inner_mut()
                .seL4_TCB_BindNotification(cptr.bits(), notification.bits())
        }))
    }
}

impl<C: InvocationContext> RelativeCPtr<C> {
    pub fn revoke(self) -> Result<()> {
        Error::wrap(self.invoke(|cptr, path, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_CNode_Revoke(
                cptr.bits(),
                path.bits(),
                path.depth_for_kernel(),
            )
        }))
    }

    pub fn delete(self) -> Result<()> {
        Error::wrap(self.invoke(|cptr, path, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_CNode_Delete(
                cptr.bits(),
                path.bits(),
                path.depth_for_kernel(),
            )
        }))
    }

    pub fn copy(self, src: &RelativeCPtr, rights: CapRights) -> Result<()> {
        Error::wrap(self.invoke(|cptr, path, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_CNode_Copy(
                cptr.bits(),
                path.bits(),
                path.depth_for_kernel(),
                src.root().bits(),
                src.path().bits(),
                src.path().depth_for_kernel(),
                rights.into_inner(),
            )
        }))
    }

    pub fn mint(self, src: &RelativeCPtr, rights: CapRights, badge: Word) -> Result<()> {
        Error::wrap(self.invoke(|cptr, path, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_CNode_Mint(
                cptr.bits(),
                path.bits(),
                path.depth_for_kernel(),
                src.root().bits(),
                src.path().bits(),
                src.path().depth_for_kernel(),
                rights.into_inner(),
                badge,
            )
        }))
    }

    pub fn mutate(self, src: &RelativeCPtr, badge: Word) -> Result<()> {
        Error::wrap(self.invoke(|cptr, path, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_CNode_Mutate(
                cptr.bits(),
                path.bits(),
                path.depth_for_kernel(),
                src.root().bits(),
                src.path().bits(),
                src.path().depth_for_kernel(),
                badge,
            )
        }))
    }

    pub fn save_caller(self) -> Result<()> {
        Error::wrap(self.invoke(|cptr, path, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_CNode_SaveCaller(
                cptr.bits(),
                path.bits(),
                path.depth_for_kernel(),
            )
        }))
    }
}
