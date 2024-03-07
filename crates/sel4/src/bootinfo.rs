//
// Copyright 2023, Colias Group, LLC
// Copyright (c) 2020 Arm Limited
//
// SPDX-License-Identifier: MIT
//

#![allow(clippy::useless_conversion)]

use core::mem;
use core::ops::{Deref, Range};
use core::slice;

use sel4_config::sel4_cfg;

use crate::{cap_type, init_thread::SlotRegion, newtype_methods, sys, FrameSize, IpcBuffer};

/// A wrapped pointer to a [`BootInfo`] block.
///
/// Access [`BootInfo`] via `Deref`, and [`BootInfoExtraIter`] via [`extra`](BootInfoPtr::extra).
#[repr(transparent)]
#[derive(Debug)]
pub struct BootInfoPtr {
    ptr: *const BootInfo,
}

impl BootInfoPtr {
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn new(ptr: *const BootInfo) -> Self {
        assert_eq!(ptr.cast::<()>().align_offset(FrameSize::GRANULE.bytes()), 0); // sanity check
        Self { ptr }
    }

    pub fn ptr(&self) -> *const BootInfo {
        self.ptr
    }

    fn extra_ptr(&self) -> *const u8 {
        self.ptr
            .cast::<u8>()
            .wrapping_offset(Self::EXTRA_OFFSET.try_into().unwrap())
    }

    fn extra_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.extra_ptr(), self.extra_len()) }
    }

    pub fn extra(&self) -> BootInfoExtraIter {
        BootInfoExtraIter::new(self)
    }

    pub fn footprint_size(&self) -> usize {
        Self::EXTRA_OFFSET + self.extra_len()
    }

    const EXTRA_OFFSET: usize = FrameSize::GRANULE.bytes();
}

impl Deref for BootInfoPtr {
    type Target = BootInfo;

    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr().as_ref().unwrap() }
    }
}

/// Corresponds to `seL4_BootInfo`.
#[repr(transparent)]
#[derive(Debug)]
pub struct BootInfo(sys::seL4_BootInfo);

impl BootInfo {
    newtype_methods!(pub sys::seL4_BootInfo);

    fn extra_len(&self) -> usize {
        self.inner().extraLen.try_into().unwrap()
    }

    pub fn ipc_buffer(&self) -> *mut IpcBuffer {
        self.inner().ipcBuffer.cast()
    }

    pub fn empty(&self) -> SlotRegion<cap_type::Null> {
        SlotRegion::from_sys(self.inner().empty)
    }

    pub fn user_image_frames(&self) -> SlotRegion<cap_type::Granule> {
        SlotRegion::from_sys(self.inner().userImageFrames)
    }

    #[sel4_cfg(KERNEL_MCS)]
    pub fn sched_control(&self) -> SlotRegion<cap_type::SchedControl> {
        SlotRegion::from_sys(self.inner().schedcontrol)
    }

    pub fn untyped(&self) -> SlotRegion<cap_type::Untyped> {
        SlotRegion::from_sys(self.inner().untyped)
    }

    fn untyped_list_inner(&self) -> &[sys::seL4_UntypedDesc] {
        &self.inner().untypedList[..self.untyped().len()]
    }

    pub fn untyped_list(&self) -> &[UntypedDesc] {
        let inner = self.untyped_list_inner();
        // safe because of #[repr(trasnparent)]
        unsafe { slice::from_raw_parts(inner.as_ptr().cast(), inner.len()) }
    }

    fn untyped_list_partition_point(&self) -> usize {
        self.untyped_list().partition_point(|ut| ut.is_device())
    }

    pub fn device_untyped_range(&self) -> Range<usize> {
        0..self.untyped_list_partition_point()
    }

    pub fn kernel_untyped_range(&self) -> Range<usize> {
        self.untyped_list_partition_point()..self.untyped_list().len()
    }
}

/// Corresponds to `seL4_UntypedDesc`.
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UntypedDesc(sys::seL4_UntypedDesc);

impl UntypedDesc {
    newtype_methods!(pub sys::seL4_UntypedDesc);

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

/// An iterator for accessing the [`BootInfoExtra`] entires associated with a [`BootInfoPtr`].
pub struct BootInfoExtraIter<'a> {
    bootinfo: &'a BootInfoPtr,
    cursor: usize,
}

impl<'a> BootInfoExtraIter<'a> {
    fn new(bootinfo: &'a BootInfoPtr) -> Self {
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
            let header = {
                let mut it = self.bootinfo.extra_slice()[self.cursor..]
                    .chunks(mem::size_of::<sys::seL4_Word>());
                let mut munch_word =
                    || sys::seL4_Word::from_ne_bytes(it.next().unwrap().try_into().unwrap());
                let id = munch_word();
                let len = munch_word();
                sys::seL4_BootInfoHeader { id, len }
            };
            let id = BootInfoExtraId::from_sys(header.id);
            let len = usize::try_from(header.len).unwrap();
            let content_with_header_start = self.cursor;
            let content_with_header_end = content_with_header_start + len;
            self.cursor = content_with_header_end;
            if let Some(id) = id {
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
