use core::mem;

use sel4_config::sel4_cfg;

use crate::{
    local_cptr::*, CNodeCapData, CPtr, CapRights, Error, ObjectBlueprint, RelativeCPtr, Result,
    UserContext, Word, IPC_BUFFER,
};

// NOTE
// &self enables convenient use of Deref at the cost of indirection. Is this appropriate?

impl Untyped {
    pub fn retype(
        &self,
        blueprint: &ObjectBlueprint,
        dst: &RelativeCPtr,
        dst_offset: usize,
        num_objects: usize,
    ) -> Result<()> {
        Error::wrap(IPC_BUFFER.borrow_mut().seL4_Untyped_Retype(
            self.bits(),
            blueprint.ty().into_sys().try_into().unwrap(),
            blueprint.api_size_bits().unwrap_or(0).try_into().unwrap(),
            dst.root().bits(),
            dst.path().bits(),
            dst.path().depth().try_into().unwrap(),
            dst_offset.try_into().unwrap(),
            num_objects.try_into().unwrap(),
        ))
    }
}

const USER_CONTEXT_MAX_REG_COUNT: usize = mem::size_of::<UserContext>() / mem::size_of::<Word>();

impl TCB {
    pub fn read_registers(&self, suspend: bool, count: Word) -> Result<UserContext> {
        let mut regs: UserContext = Default::default();
        let err = IPC_BUFFER.borrow_mut().seL4_TCB_ReadRegisters(
            self.bits(),
            suspend.into(),
            0,
            count,
            regs.inner_mut(),
        );
        Error::or(err, regs)
    }

    pub fn read_all_registers(&self, suspend: bool) -> Result<UserContext> {
        self.read_registers(suspend, USER_CONTEXT_MAX_REG_COUNT.try_into().unwrap())
    }

    // HACK should not be mut
    pub fn write_registers(&self, resume: bool, count: Word, regs: &mut UserContext) -> Result<()> {
        Error::wrap(IPC_BUFFER.borrow_mut().seL4_TCB_WriteRegisters(
            self.bits(),
            resume.into(),
            0,
            count,
            regs.inner_mut(),
        ))
    }

    pub fn write_all_registers(&self, resume: bool, regs: &mut UserContext) -> Result<()> {
        self.write_registers(resume, USER_CONTEXT_MAX_REG_COUNT.try_into().unwrap(), regs)
    }

    pub fn resume(&self) -> Result<()> {
        Error::wrap(IPC_BUFFER.borrow_mut().seL4_TCB_Resume(self.bits()))
    }

    pub fn suspend(&self) -> Result<()> {
        Error::wrap(IPC_BUFFER.borrow_mut().seL4_TCB_Suspend(self.bits()))
    }

    pub fn configure(
        &self,
        fault_ep: CPtr,
        cspace_root: CNode,
        cspace_root_data: CNodeCapData,
        vspace_root: PGD,
        ipc_buffer: Word,
        ipc_buffer_frame: SmallPage,
    ) -> Result<()> {
        Error::wrap(IPC_BUFFER.borrow_mut().seL4_TCB_Configure(
            self.bits(),
            fault_ep.bits(),
            cspace_root.bits(),
            cspace_root_data.into_word(),
            vspace_root.bits(),
            0, /* HACK */
            ipc_buffer,
            ipc_buffer_frame.bits(),
        ))
    }

    pub fn set_sched_params(&self, authority: TCB, mcp: Word, priority: Word) -> Result<()> {
        Error::wrap(IPC_BUFFER.borrow_mut().seL4_TCB_SetSchedParams(
            self.bits(),
            authority.bits(),
            mcp,
            priority,
        ))
    }

    #[sel4_cfg(not(MAX_NUM_NODES = "1"))]
    pub fn set_affinity(&self, affinity: Word) -> Result<()> {
        Error::wrap(
            IPC_BUFFER
                .borrow_mut()
                .seL4_TCB_SetAffinity(self.bits(), affinity),
        )
    }

    pub fn set_tls_base(&self, tls_base: Word) -> Result<()> {
        Error::wrap(
            IPC_BUFFER
                .borrow_mut()
                .seL4_TCB_SetTLSBase(self.bits(), tls_base),
        )
    }

    pub fn bind_notification(&self, notification: Notification) -> Result<()> {
        Error::wrap(
            IPC_BUFFER
                .borrow_mut()
                .seL4_TCB_BindNotification(self.bits(), notification.bits()),
        )
    }
}

impl RelativeCPtr {
    pub fn revoke(&self) -> Result<()> {
        Error::wrap(IPC_BUFFER.borrow_mut().seL4_CNode_Revoke(
            self.root().bits(),
            self.path().bits(),
            self.path().depth_for_kernel(),
        ))
    }

    pub fn delete(&self) -> Result<()> {
        Error::wrap(IPC_BUFFER.borrow_mut().seL4_CNode_Delete(
            self.root().bits(),
            self.path().bits(),
            self.path().depth_for_kernel(),
        ))
    }

    pub fn copy(&self, src: &RelativeCPtr, rights: CapRights) -> Result<()> {
        Error::wrap(IPC_BUFFER.borrow_mut().seL4_CNode_Copy(
            self.root().bits(),
            self.path().bits(),
            self.path().depth_for_kernel(),
            src.root().bits(),
            src.path().bits(),
            src.path().depth_for_kernel(),
            rights.into_inner(),
        ))
    }

    pub fn mint(&self, src: &RelativeCPtr, rights: CapRights, badge: Word) -> Result<()> {
        Error::wrap(IPC_BUFFER.borrow_mut().seL4_CNode_Mint(
            self.root().bits(),
            self.path().bits(),
            self.path().depth_for_kernel(),
            src.root().bits(),
            src.path().bits(),
            src.path().depth_for_kernel(),
            rights.into_inner(),
            badge,
        ))
    }

    pub fn mutate(&self, src: &RelativeCPtr, badge: Word) -> Result<()> {
        Error::wrap(IPC_BUFFER.borrow_mut().seL4_CNode_Mutate(
            self.root().bits(),
            self.path().bits(),
            self.path().depth_for_kernel(),
            src.root().bits(),
            src.path().bits(),
            src.path().depth_for_kernel(),
            badge,
        ))
    }

    pub fn save_caller(&self) -> Result<()> {
        Error::wrap(IPC_BUFFER.borrow_mut().seL4_CNode_SaveCaller(
            self.root().bits(),
            self.path().bits(),
            self.path().depth_for_kernel(),
        ))
    }
}
