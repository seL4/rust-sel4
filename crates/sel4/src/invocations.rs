//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

#![allow(clippy::useless_conversion)]

use core::mem;

use sel4_config::{sel4_cfg, sel4_cfg_if};

use crate::{
    cap::*, sys, AbsoluteCPtr, CNodeCapData, CPtr, CapRights, Error, InvocationContext,
    ObjectBlueprint, Result, UserContext, Word,
};

#[sel4_cfg(KERNEL_MCS)]
use crate::Badge;

/// Corresponds to `seL4_Time`.
#[sel4_cfg(KERNEL_MCS)]
pub type Time = u64;

impl<C: InvocationContext> Untyped<C> {
    /// Corresponds to `seL4_Untyped_Retype`.
    pub fn untyped_retype(
        self,
        blueprint: &ObjectBlueprint,
        dst: &AbsoluteCPtr,
        dst_offset: usize,
        num_objects: usize,
    ) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_Untyped_Retype(
                cptr.bits(),
                blueprint.ty().into_sys().into(),
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

const USER_CONTEXT_MAX_REG_COUNT: usize =
    mem::size_of::<sys::seL4_UserContext>() / mem::size_of::<Word>();

impl<C: InvocationContext> Tcb<C> {
    /// Corresponds to `seL4_TCB_ReadRegisters`.
    pub fn tcb_read_registers(self, suspend: bool, count: Word) -> Result<UserContext> {
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

    pub fn tcb_read_all_registers(self, suspend: bool) -> Result<UserContext> {
        self.tcb_read_registers(suspend, USER_CONTEXT_MAX_REG_COUNT.try_into().unwrap())
    }

    /// Corresponds to `seL4_TCB_WriteRegisters`.
    // HACK should not be mut
    pub fn tcb_write_registers(
        self,
        resume: bool,
        count: Word,
        regs: &mut UserContext,
    ) -> Result<()> {
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

    pub fn tcb_write_all_registers(self, resume: bool, regs: &mut UserContext) -> Result<()> {
        self.tcb_write_registers(resume, USER_CONTEXT_MAX_REG_COUNT.try_into().unwrap(), regs)
    }

    /// Corresponds to `seL4_TCB_Resume`.
    pub fn tcb_resume(self) -> Result<()> {
        Error::wrap(
            self.invoke(|cptr, ipc_buffer| ipc_buffer.inner_mut().seL4_TCB_Resume(cptr.bits())),
        )
    }

    /// Corresponds to `seL4_TCB_Suspend`.
    pub fn tcb_suspend(self) -> Result<()> {
        Error::wrap(
            self.invoke(|cptr, ipc_buffer| ipc_buffer.inner_mut().seL4_TCB_Suspend(cptr.bits())),
        )
    }

    sel4_cfg_if! {
        if #[sel4_cfg(KERNEL_MCS)] {
            /// Corresponds to `seL4_TCB_Configure`.
            pub fn tcb_configure(
                self,
                cspace_root: CNode,
                cspace_root_data: CNodeCapData,
                vspace_root: VSpace,
                ipc_buffer: Word,
                ipc_buffer_frame: Granule,
            ) -> Result<()> {
                Error::wrap(self.invoke(|cptr, ctx_ipc_buffer| {
                    ctx_ipc_buffer.inner_mut().seL4_TCB_Configure(
                        cptr.bits(),
                        cspace_root.bits(),
                        cspace_root_data.into_word(),
                        vspace_root.bits(),
                        0, /* HACK */
                        ipc_buffer,
                        ipc_buffer_frame.bits(),
                    )
                }))
            }
        } else {
            /// Corresponds to `seL4_TCB_Configure`.
            pub fn tcb_configure(
                self,
                fault_ep: CPtr,
                cspace_root: CNode,
                cspace_root_data: CNodeCapData,
                vspace_root: VSpace,
                ipc_buffer: Word,
                ipc_buffer_frame: Granule,
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
        }
    }

    /// Corresponds to `seL4_TCB_SetSpace`.
    pub fn tcb_set_space(
        self,
        fault_ep: CPtr,
        cspace_root: CNode,
        cspace_root_data: CNodeCapData,
        vspace_root: VSpace,
    ) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_TCB_SetSpace(
                cptr.bits(),
                fault_ep.bits(),
                cspace_root.bits(),
                cspace_root_data.into_word(),
                vspace_root.bits(),
                0, /* HACK */
            )
        }))
    }

    sel4_cfg_if! {
        if #[sel4_cfg(KERNEL_MCS)] {
            /// Corresponds to `seL4_TCB_SetSchedParams`.
            pub fn tcb_set_sched_params(
                self,
                authority: Tcb,
                mcp: Word,
                priority: Word,
                sched_context: SchedContext,
                fault_ep: Endpoint,
            ) -> Result<()> {
                Error::wrap(self.invoke(|cptr, ipc_buffer| {
                    ipc_buffer.inner_mut().seL4_TCB_SetSchedParams(
                        cptr.bits(),
                        authority.bits(),
                        mcp,
                        priority,
                        sched_context.bits(),
                        fault_ep.bits(),
                    )
                }))
            }
        } else {
            /// Corresponds to `seL4_TCB_SetSchedParams`.
            pub fn tcb_set_sched_params(self, authority: Tcb, mcp: Word, priority: Word) -> Result<()> {
                Error::wrap(self.invoke(|cptr, ipc_buffer| {
                    ipc_buffer.inner_mut().seL4_TCB_SetSchedParams(
                        cptr.bits(),
                        authority.bits(),
                        mcp,
                        priority,
                    )
                }))
            }
        }
    }

    #[sel4_cfg(KERNEL_MCS)]
    pub fn tcb_set_timeout_endpoint(self, timeout_endpoint: Endpoint) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer
                .inner_mut()
                .seL4_TCB_SetTimeoutEndpoint(cptr.bits(), timeout_endpoint.bits())
        }))
    }

    /// Corresponds to `seL4_TCB_SetAffinity`.
    #[sel4_cfg(all(not(KERNEL_MCS), not(MAX_NUM_NODES = "1")))]
    pub fn tcb_set_affinity(self, affinity: Word) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer
                .inner_mut()
                .seL4_TCB_SetAffinity(cptr.bits(), affinity)
        }))
    }

    /// Corresponds to `seL4_TCB_SetTLSBase`.
    pub fn tcb_set_tls_base(self, tls_base: Word) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer
                .inner_mut()
                .seL4_TCB_SetTLSBase(cptr.bits(), tls_base)
        }))
    }

    /// Corresponds to `seL4_TCB_BindNotification`.
    pub fn tcb_bind_notification(self, notification: Notification) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer
                .inner_mut()
                .seL4_TCB_BindNotification(cptr.bits(), notification.bits())
        }))
    }

    /// Corresponds to `seL4_TCB_UnbindNotification`.
    pub fn tcb_unbind_notification(self) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer
                .inner_mut()
                .seL4_TCB_UnbindNotification(cptr.bits())
        }))
    }
}

#[sel4_cfg(KERNEL_MCS)]
impl<C: InvocationContext> SchedControl<C> {
    /// Corresponds to `seL4_SchedControl_ConfigureFlags`.
    pub fn sched_control_configure_flags(
        self,
        sched_context: SchedContext,
        budget: Time,
        period: Time,
        extra_refills: Word,
        badge: Badge,
        flags: Word,
    ) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_SchedControl_ConfigureFlags(
                cptr.bits(),
                sched_context.bits(),
                budget,
                period,
                extra_refills,
                badge,
                flags,
            )
        }))
    }
}

impl<C: InvocationContext> IrqControl<C> {
    /// Corresponds to `seL4_IRQControl_Get`.
    pub fn irq_control_get(self, irq: Word, dst: &AbsoluteCPtr) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_IRQControl_Get(
                cptr.bits(),
                irq,
                dst.root().bits(),
                dst.path().bits(),
                dst.path().depth_for_kernel(),
            )
        }))
    }
}

impl<C: InvocationContext> IrqHandler<C> {
    /// Corresponds to `seL4_IRQHandler_Ack`.
    pub fn irq_handler_ack(self) -> Result<()> {
        Error::wrap(
            self.invoke(|cptr, ipc_buffer| ipc_buffer.inner_mut().seL4_IRQHandler_Ack(cptr.bits())),
        )
    }

    /// Corresponds to `seL4_IRQHandler_SetNotification`.
    pub fn irq_handler_set_notification(self, notification: Notification) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer
                .inner_mut()
                .seL4_IRQHandler_SetNotification(cptr.bits(), notification.bits())
        }))
    }

    /// Corresponds to `seL4_IRQHandler_Clear`.
    pub fn irq_handler_clear(self) -> Result<()> {
        Error::wrap(
            self.invoke(|cptr, ipc_buffer| {
                ipc_buffer.inner_mut().seL4_IRQHandler_Clear(cptr.bits())
            }),
        )
    }
}

impl<C: InvocationContext> AbsoluteCPtr<C> {
    /// Corresponds to `seL4_CNode_Revoke`.
    pub fn revoke(self) -> Result<()> {
        Error::wrap(self.invoke(|cptr, path, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_CNode_Revoke(
                cptr.bits(),
                path.bits(),
                path.depth_for_kernel(),
            )
        }))
    }

    /// Corresponds to `seL4_CNode_Delete`.
    pub fn delete(self) -> Result<()> {
        Error::wrap(self.invoke(|cptr, path, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_CNode_Delete(
                cptr.bits(),
                path.bits(),
                path.depth_for_kernel(),
            )
        }))
    }

    /// Corresponds to `seL4_CNode_Copy`.
    pub fn copy(self, src: &AbsoluteCPtr, rights: CapRights) -> Result<()> {
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

    /// Corresponds to `seL4_CNode_Mint`.
    pub fn mint(self, src: &AbsoluteCPtr, rights: CapRights, badge: Word) -> Result<()> {
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

    /// Corresponds to `seL4_CNode_Move`.
    pub fn move_(self, src: &AbsoluteCPtr) -> Result<()> {
        Error::wrap(self.invoke(|cptr, path, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_CNode_Move(
                cptr.bits(),
                path.bits(),
                path.depth_for_kernel(),
                src.root().bits(),
                src.path().bits(),
                src.path().depth_for_kernel(),
            )
        }))
    }

    /// Corresponds to `seL4_CNode_Mutate`.
    pub fn mutate(self, src: &AbsoluteCPtr, badge: Word) -> Result<()> {
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

    /// Corresponds to `seL4_CNode_SaveCaller`.
    #[sel4_cfg(not(KERNEL_MCS))]
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
