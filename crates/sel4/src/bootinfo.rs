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

use crate::{FrameObjectType, IpcBuffer, cap_type, init_thread::SlotRegion, newtype_methods, sys};

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
        assert_eq!(
            ptr.cast::<()>()
                .align_offset(FrameObjectType::GRANULE.bytes()),
            0
        ); // sanity check
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

    pub fn extra(&self) -> BootInfoExtraIter<'_> {
        BootInfoExtraIter::new(self)
    }

    pub fn footprint_size(&self) -> usize {
        Self::EXTRA_OFFSET + self.extra_len()
    }

    const EXTRA_OFFSET: usize = FrameObjectType::GRANULE.bytes();
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
#[repr(C)]
pub struct BootInfoExtra<'a> {
    pub header: BootInfoHeader,
    pub chunk: &'a [u8],
}

/// Common header for all additional bootinfo chunks.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BootInfoHeader {
    id: sys::seL4_BootInfoID::Type,
    len: sys::seL4_Word,
}

impl BootInfoHeader {
    /// Create a new `BootInfoHeader`.
    pub fn new(id: BootInfoExtraId, len: sys::seL4_Word) -> Self {
        let id = id as sys::seL4_BootInfoID::Type;
        Self { id, len }
    }

    /// Get the ID of this extra bootinfo chunk.
    pub fn id(&self) -> BootInfoExtraId {
        BootInfoExtraId::from_sys(self.id).unwrap()
    }

    /// Get the raw ID value of this extra bootinfo chunk.
    pub fn id_raw(&self) -> sys::seL4_BootInfoID::Type {
        self.id
    }

    /// Get the length of this extra bootinfo chunk, including the header.
    pub fn len(&self) -> usize {
        self.len.try_into().unwrap()
    }

    /// Get the length of the payload of this extra bootinfo chunk, excluding the header.
    pub fn payload_len(&self) -> usize {
        self.len()
            .saturating_sub(mem::size_of::<Self>().try_into().unwrap())
            .try_into()
            .unwrap()
    }
}

/// Corresponds to `seL4_BootInfoID`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum BootInfoExtraId {
    Padding,
    X86VBE,          // VESA BIOS Extensions info
    X86MBMMap,       // Multiboot Memory Map
    X86_ACPI_RSDP,   // ACPI Root System Description Pointer
    X86_Framebuffer, // Framebuffer info
    X86_TSC_Freq,    // TSC frequency (in MHz)
    FDT,             // Flattened Device Tree
}

impl BootInfoExtraId {
    pub fn from_sys(id: sys::seL4_BootInfoID::Type) -> Option<Self> {
        use sys::seL4_BootInfoID::*;
        match id {
            SEL4_BOOTINFO_HEADER_PADDING => Some(Self::Padding),
            SEL4_BOOTINFO_HEADER_X86_VBE => Some(Self::X86VBE),
            SEL4_BOOTINFO_HEADER_X86_MBMMAP => Some(Self::X86MBMMap),
            SEL4_BOOTINFO_HEADER_X86_ACPI_RSDP => Some(Self::X86_ACPI_RSDP),
            SEL4_BOOTINFO_HEADER_X86_FRAMEBUFFER => Some(Self::X86_Framebuffer),
            SEL4_BOOTINFO_HEADER_X86_TSC_FREQ => Some(Self::X86_TSC_Freq),
            SEL4_BOOTINFO_HEADER_FDT => Some(Self::FDT),
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
        let slice = self.bootinfo.extra_slice();
        const HEADER_SIZE: usize = mem::size_of::<BootInfoHeader>();

        let header: &BootInfoHeader = {
            // SAFETY: Bounds check before dereferencing
            if self.cursor + HEADER_SIZE > slice.len() {
                return None;
            }

            let bytes = &slice[self.cursor..self.cursor + HEADER_SIZE];
            unsafe { &*(bytes.as_ptr() as *const BootInfoHeader) }
        };

        let chunk = {
            let chunk_start = self.cursor + HEADER_SIZE;
            let chunk_end = chunk_start + header.payload_len();

            // SAFETY: Bounds check before dereferencing
            if chunk_end > slice.len() || header.len() < HEADER_SIZE {
                return None;
            }

            &slice[chunk_start..chunk_end]
        };

        self.cursor += header.len();
        Some(BootInfoExtra {
            header: *header,
            chunk,
        })
    }
}
