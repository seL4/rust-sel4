//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use crate::{
    cap::*, cap_type, sel4_cfg_wrap_match, AbsoluteCPtr, Cap, CapRights, CapTypeForFrameObject,
    Error, InvocationContext, Result, TranslationStructureObjectType, VmAttributes, Word,
};

impl<T: CapTypeForFrameObject, C: InvocationContext> Cap<T, C> {
    /// Corresponds to `seL4_X86_Page_Map`.
    pub fn frame_map(
        self,
        vspace: VSpace,
        vaddr: usize,
        rights: CapRights,
        attrs: VmAttributes,
    ) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_X86_Page_Map(
                cptr.bits(),
                vspace.bits(),
                vaddr.try_into().unwrap(),
                rights.into_inner(),
                attrs.into_inner(),
            )
        }))
    }

    /// Corresponds to `seL4_X86_Page_Unmap`.
    pub fn frame_unmap(self) -> Result<()> {
        Error::wrap(
            self.invoke(|cptr, ipc_buffer| ipc_buffer.inner_mut().seL4_X86_Page_Unmap(cptr.bits())),
        )
    }

    /// Corresponds to `seL4_X86_Page_GetAddress`.
    pub fn frame_get_address(self) -> Result<usize> {
        let ret = self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_X86_Page_GetAddress(cptr.bits())
        });
        match Error::from_sys(ret.error) {
            None => Ok(ret.paddr.try_into().unwrap()),
            Some(err) => Err(err),
        }
    }
}

impl<C: InvocationContext> PDPT<C> {
    pub fn pdpt_map(self, vspace: VSpace, vaddr: usize, attr: VmAttributes) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_X86_PDPT_Map(
                cptr.bits(),
                vspace.bits(),
                vaddr.try_into().unwrap(),
                attr.into_inner(),
            )
        }))
    }
}

impl<C: InvocationContext> PageDirectory<C> {
    pub fn page_directory_map(
        self,
        vspace: VSpace,
        vaddr: usize,
        attr: VmAttributes,
    ) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_X86_PageDirectory_Map(
                cptr.bits(),
                vspace.bits(),
                vaddr.try_into().unwrap(),
                attr.into_inner(),
            )
        }))
    }
}

impl<C: InvocationContext> PageTable<C> {
    pub fn page_table_map(self, vspace: VSpace, vaddr: usize, attr: VmAttributes) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_X86_PageTable_Map(
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
        ty: TranslationStructureObjectType,
        vspace: VSpace,
        vaddr: usize,
        attr: VmAttributes,
    ) -> Result<()> {
        sel4_cfg_wrap_match! {
            match ty {
                #[sel4_cfg(ARCH_X86_64)]
                TranslationStructureObjectType::PDPT => self.cast::<cap_type::PDPT>().pdpt_map(vspace, vaddr, attr),
                TranslationStructureObjectType::PageDirectory => self
                    .cast::<cap_type::PageDirectory>()
                    .page_directory_map(vspace, vaddr, attr),
                TranslationStructureObjectType::PageTable => self
                    .cast::<cap_type::PageTable>()
                    .page_table_map(vspace, vaddr, attr),
                _ => panic!(),
            }
        }
    }
}

impl<C: InvocationContext> IrqControl<C> {
    /// Corresponds to `seL4_IRQControl_GetIOAPIC`.
    pub fn irq_control_get_ioapic(
        self,
        ioapic: Word,
        pin: Word,
        level: Word,
        polarity: Word,
        vector: Word,
        dst: &AbsoluteCPtr,
    ) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_IRQControl_GetIOAPIC(
                cptr.bits(),
                dst.root().bits(),
                dst.path().bits(),
                dst.path().depth_for_kernel(),
                ioapic,
                pin,
                level,
                polarity,
                vector,
            )
        }))
    }

    /// Corresponds to `seL4_IRQControl_GetMSI`.
    pub fn irq_control_get_msi(
        self,
        pci_bus: Word,
        pci_dev: Word,
        pci_func: Word,
        handle: Word,
        vector: Word,
        dst: &AbsoluteCPtr,
    ) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_IRQControl_GetMSI(
                cptr.bits(),
                dst.root().bits(),
                dst.path().bits(),
                dst.path().depth_for_kernel(),
                pci_bus,
                pci_dev,
                pci_func,
                handle,
                vector,
            )
        }))
    }
}

impl<C: InvocationContext> IOPortControl<C> {
    /// Corresponds to `seL4_X86_IOPortControl_Issue`.
    pub fn ioport_control_issue(
        self,
        first_port: Word,
        last_port: Word,
        dst: &AbsoluteCPtr,
    ) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_X86_IOPortControl_Issue(
                cptr.bits(),
                first_port,
                last_port,
                dst.root().bits(),
                dst.path().bits(),
                dst.path().depth_for_kernel(),
            )
        }))
    }
}

impl<C: InvocationContext> AsidControl<C> {
    /// Corresponds to `seL4_X86_ASIDControl_MakePool`.
    pub fn asid_control_make_pool(self, untyped: Untyped, dst: &AbsoluteCPtr) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer.inner_mut().seL4_X86_ASIDControl_MakePool(
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
    /// Corresponds to `seL4_X86_ASIDPool_Assign`.
    pub fn asid_pool_assign(self, vspace: VSpace) -> Result<()> {
        Error::wrap(self.invoke(|cptr, ipc_buffer| {
            ipc_buffer
                .inner_mut()
                .seL4_X86_ASIDPool_Assign(cptr.bits(), vspace.bits())
        }))
    }
}
