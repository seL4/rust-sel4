//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use crate::{
    cap::*, cap_type, AbsoluteCPtr, Cap, CapRights, CapTypeForFrameObject, Error,
    InvocationContext, Result, VmAttributes,
};

impl<T: CapTypeForFrameObject, C: InvocationContext> Cap<T, C> {
    /// Corresponds to `seL4_RISCV_Page_Map`.
    pub fn frame_map(
        self,
        page_table: PageTable,
        vaddr: usize,
        rights: CapRights,
        attrs: VmAttributes,
    ) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_RISCV_Page_Map(
                cptr.bits(),
                page_table.bits(),
                vaddr.try_into().unwrap(),
                rights.into_inner(),
                attrs.into_inner(),
            )
        }))
    }

    /// Corresponds to `seL4_RISCV_Page_Unmap`.
    pub fn frame_unmap(self) -> Result<()> {
        Error::wrap(
            self.invoke(|cptr, ipc_buffer| {
                ipc_buffer.inner_mut().seL4_RISCV_Page_Unmap(cptr.bits())
            }),
        )
    }

    /// Corresponds to `seL4_RISCV_Page_GetAddress`.
    pub fn frame_get_address(self) -> Result<usize> {
        let ret = self.invoke(|cptr, ipc_buffer| {
            ipc_buffer
                .inner_mut()
                .seL4_RISCV_Page_GetAddress(cptr.bits())
        });
        match Error::from_sys(ret.error) {
            None => Ok(ret.paddr.try_into().unwrap()),
            Some(err) => Err(err),
        }
    }
}

impl<C: InvocationContext> PageTable<C> {
    pub fn page_table_map(self, vspace: PageTable, vaddr: usize, attr: VmAttributes) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_RISCV_PageTable_Map(
                cptr.bits(),
                vspace.bits(),
                vaddr.try_into().unwrap(),
                attr.into_inner(),
            )
        }))
    }
}

impl<C: InvocationContext> UnspecifiedIntermediateTranslationStructure<C> {
    pub fn generic_intermediate_translation_structure_map(
        self,
        vspace: VSpace,
        _level: usize,
        vaddr: usize,
        attr: VmAttributes,
    ) -> Result<()> {
        self.cast::<cap_type::PageTable>()
            .page_table_map(vspace, vaddr, attr)
    }
}

impl<C: InvocationContext> AsidControl<C> {
    /// Corresponds to `seL4_RISCV_ASIDControl_MakePool`.
    pub fn asid_control_make_pool(self, untyped: Untyped, dst: &AbsoluteCPtr) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_RISCV_ASIDControl_MakePool(
                cptr.bits(),
                untyped.bits(),
                dst.root().bits(),
                dst.path().bits(),
                dst.path().depth_for_kernel(),
            )
        }))
    }
}

impl<C: InvocationContext> AsidPool<C> {
    /// Corresponds to `seL4_RISCV_ASIDPool_Assign`.
    pub fn asid_pool_assign(self, vspace: PageTable) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer
                .inner_mut()
                .seL4_RISCV_ASIDPool_Assign(cptr.bits(), vspace.bits())
        }))
    }
}
