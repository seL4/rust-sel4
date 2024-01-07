//
// Copyright 2023, Colias Group, LLC
// Copyright (c) 2020 Arm Limited
//
// SPDX-License-Identifier: MIT
//

#![allow(clippy::useless_conversion)]

use core::mem;
use core::ops::Range;
use core::slice;

use crate::{
    newtype_methods, sel4_cfg, sys, ASIDControl, ASIDPool, CNode, CPtr, CapType, IPCBuffer,
    IRQControl, LocalCPtr, Null, VSpace, GRANULE_SIZE, TCB,
};

#[sel4_cfg(KERNEL_MCS)]
use crate::SchedControl;

/// Corresponds to `seL4_BootInfo`.
#[derive(Debug)]
pub struct BootInfo {
    ptr: *const sys::seL4_BootInfo,
}

impl BootInfo {
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn from_ptr(ptr: *const sys::seL4_BootInfo) -> Self {
        assert_eq!(ptr.addr() % GRANULE_SIZE.bytes(), 0); // sanity check
        Self { ptr }
    }

    pub fn ptr(&self) -> *const sys::seL4_BootInfo {
        self.ptr
    }

    pub fn inner(&self) -> &sys::seL4_BootInfo {
        unsafe { self.ptr().as_ref().unwrap() }
    }

    fn extra_ptr(&self) -> *const u8 {
        unsafe {
            self.ptr
                .cast::<u8>()
                .offset(GRANULE_SIZE.bytes().try_into().unwrap())
        }
    }

    fn extra_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.extra_ptr(), self.extra_len()) }
    }

    pub fn extra(&self) -> BootInfoExtraIter {
        BootInfoExtraIter::new(self)
    }

    pub fn extra_len(&self) -> usize {
        self.inner().extraLen.try_into().unwrap()
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn ipc_buffer(&self) -> IPCBuffer {
        IPCBuffer::from_ptr(self.inner().ipcBuffer)
    }

    pub fn empty(&self) -> Range<InitCSpaceSlot> {
        region_to_range(self.inner().empty)
    }

    pub fn user_image_frames(&self) -> Range<InitCSpaceSlot> {
        region_to_range(self.inner().userImageFrames)
    }

    #[sel4_cfg(KERNEL_MCS)]
    pub fn sched_control_slot(&self, node: usize) -> InitCSpaceSlot {
        let range = region_to_range(self.inner().schedcontrol);
        assert!(node < range.len());
        range.start + node
    }

    #[sel4_cfg(KERNEL_MCS)]
    pub fn sched_control(&self, node: usize) -> SchedControl {
        Self::init_cspace_local_cptr(self.sched_control_slot(node))
    }

    pub fn untyped(&self) -> Range<InitCSpaceSlot> {
        region_to_range(self.inner().untyped)
    }

    pub fn num_untyped(&self) -> usize {
        self.untyped().end - self.untyped().start
    }

    fn untyped_list_inner(&self) -> &[sys::seL4_UntypedDesc] {
        &self.inner().untypedList[..self.num_untyped()]
    }

    pub fn untyped_list(&self) -> &[UntypedDesc] {
        let inner = self.untyped_list_inner();
        // safe because of #[repr(trasnparent)]
        unsafe { slice::from_raw_parts(inner.as_ptr().cast(), inner.len()) }
    }

    fn untyped_list_partition_point(&self) -> usize {
        self.untyped_list().partition_point(|ut| ut.is_device())
    }

    pub fn device_untyped_list(&self) -> &[UntypedDesc] {
        &self.untyped_list()[..self.untyped_list_partition_point()]
    }

    pub fn kernel_untyped_list(&self) -> &[UntypedDesc] {
        &self.untyped_list()[self.untyped_list_partition_point()..]
    }

    pub fn footprint(&self) -> Range<usize> {
        self.ptr.addr()..self.extra_ptr().addr() + self.extra_len()
    }

    pub fn init_thread_cnode() -> CNode {
        CNode::from_bits(sys::seL4_RootCapSlot::seL4_CapInitThreadCNode.into())
    }

    pub fn irq_control() -> IRQControl {
        IRQControl::from_bits(sys::seL4_RootCapSlot::seL4_CapIRQControl.into())
    }

    pub fn asid_control() -> ASIDControl {
        ASIDControl::from_bits(sys::seL4_RootCapSlot::seL4_CapASIDControl.into())
    }

    pub fn init_thread_asid_pool() -> ASIDPool {
        ASIDPool::from_bits(sys::seL4_RootCapSlot::seL4_CapInitThreadASIDPool.into())
    }

    pub fn init_thread_vspace() -> VSpace {
        VSpace::from_bits(sys::seL4_RootCapSlot::seL4_CapInitThreadVSpace.into())
    }

    pub fn init_thread_tcb() -> TCB {
        TCB::from_bits(sys::seL4_RootCapSlot::seL4_CapInitThreadTCB.into())
    }

    pub fn init_cspace_cptr(slot: InitCSpaceSlot) -> CPtr {
        CPtr::from_bits(slot.try_into().unwrap())
    }

    pub fn init_cspace_local_cptr<T: CapType>(slot: InitCSpaceSlot) -> LocalCPtr<T> {
        Self::init_cspace_cptr(slot).cast()
    }

    pub fn null() -> Null {
        Null::from_bits(0)
    }
}

/// The index of a slot in the root task's top-level CNode.
pub type InitCSpaceSlot = usize;

/// Corresponds to `seL4_UntypedDesc`.
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UntypedDesc(sys::seL4_UntypedDesc);

impl UntypedDesc {
    newtype_methods!(sys::seL4_UntypedDesc);

    pub fn paddr(&self) -> usize {
        self.inner().paddr.try_into().unwrap()
    }

    pub fn size_bits(&self) -> usize {
        self.inner().sizeBits.into()
    }

    pub fn is_device(&self) -> bool {
        self.inner().isDevice != 0
    }
}

fn region_to_range(region: sys::seL4_SlotRegion) -> Range<InitCSpaceSlot> {
    region.start.try_into().unwrap()..region.end.try_into().unwrap()
}

/// An extra bootinfo chunk along with its ID.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BootInfoExtra<'a> {
    pub id: BootInfoExtraId,
    pub content_with_header: &'a [u8],
}

impl<'a> BootInfoExtra<'a> {
    pub fn content_with_header(&self) -> &[u8] {
        self.content_with_header
    }

    pub fn content(&self) -> &[u8] {
        let content_with_header = self.content_with_header();
        &content_with_header[mem::size_of::<sys::seL4_BootInfoHeader>()..]
    }
}

/// Corresponds to `seL4_BootInfoID`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BootInfoExtraId {
    Padding,
    Fdt,
}

impl BootInfoExtraId {
    pub fn from_sys(id: sys::seL4_BootInfoID::Type) -> Option<Self> {
        match id {
            sys::seL4_BootInfoID::SEL4_BOOTINFO_HEADER_PADDING => Some(BootInfoExtraId::Padding),
            sys::seL4_BootInfoID::SEL4_BOOTINFO_HEADER_FDT => Some(BootInfoExtraId::Fdt),
            _ => None,
        }
    }
}

pub struct BootInfoExtraIter<'a> {
    bootinfo: &'a BootInfo,
    cursor: usize,
}

impl<'a> BootInfoExtraIter<'a> {
    fn new(bootinfo: &'a BootInfo) -> Self {
        Self {
            bootinfo,
            cursor: 0,
        }
    }
}

impl<'a> Iterator for BootInfoExtraIter<'a> {
    type Item = BootInfoExtra<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.cursor < self.bootinfo.extra_slice().len() {
            let header = unsafe {
                &*self
                    .bootinfo
                    .extra_slice()
                    .as_ptr()
                    .offset(self.cursor.try_into().unwrap())
                    .cast::<sys::seL4_BootInfoHeader>()
            };
            let content_with_header_start = self.cursor;
            let content_with_header_end = self.cursor + usize::try_from(header.len).unwrap();
            self.cursor = content_with_header_end;
            if let Some(id) = BootInfoExtraId::from_sys(header.id) {
                return Some(BootInfoExtra {
                    id,
                    content_with_header: &self.bootinfo.extra_slice()
                        [content_with_header_start..content_with_header_end],
                });
            }
        }
        None
    }
}
