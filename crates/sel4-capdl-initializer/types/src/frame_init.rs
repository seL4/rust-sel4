//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use alloc::boxed::Box;
use alloc::{string::String, vec::Vec};
use core::fmt;
use core::ops::Range;

#[cfg(feature = "deflate")]
use core::iter;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{SelfContained, object};

// // //

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub enum FrameInit<D, M> {
    Fill(Fill<D>),
    Embedded(M),
}

impl<D, M> FrameInit<D, M> {
    pub const fn as_fill(&self) -> Option<&Fill<D>> {
        match self {
            Self::Fill(fill) => Some(fill),
            _ => None,
        }
    }

    pub const fn as_embedded(&self) -> Option<&M> {
        match self {
            Self::Embedded(embedded) => Some(embedded),
            _ => None,
        }
    }

    pub const fn is_fill(&self) -> bool {
        self.as_fill().is_some()
    }

    pub const fn is_embedded(&self) -> bool {
        self.as_embedded().is_some()
    }
}

impl<D> FrameInit<D, NeverEmbedded> {
    #[allow(clippy::explicit_auto_deref)]
    pub const fn as_fill_infallible(&self) -> &Fill<D> {
        match self {
            Self::Fill(fill) => fill,
            Self::Embedded(absurdity) => match *absurdity {},
        }
    }
}

impl<D> object::Frame<D, NeverEmbedded> {
    pub fn can_embed(&self, granule_size_bits: usize, is_root: bool) -> bool {
        is_root
            && self.paddr.is_none()
            && self.size_bits == granule_size_bits
            && !self.init.as_fill_infallible().is_empty()
            && !self.init.as_fill_infallible().depends_on_bootinfo()
    }
}

// // //

#[derive(Copy, Clone)]
pub enum NeverEmbedded {}

// // //

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct EmbeddedFrame {
    ptr: *const u8,
}

impl EmbeddedFrame {
    pub const fn new(ptr: *const u8) -> Self {
        Self { ptr }
    }

    pub const fn ptr(&self) -> *const u8 {
        self.ptr
    }

    pub fn check(&self, frame_size: usize) {
        assert_eq!(self.ptr().cast::<()>().align_offset(frame_size), 0);
    }
}

unsafe impl Sync for EmbeddedFrame {}

pub trait SelfContainedGetEmbeddedFrame {
    fn self_contained_get_embedded_frame(&self) -> EmbeddedFrame;
}

impl SelfContainedGetEmbeddedFrame for EmbeddedFrame {
    fn self_contained_get_embedded_frame(&self) -> EmbeddedFrame {
        *self
    }
}

pub trait GetEmbeddedFrame {
    type Source: ?Sized;

    fn get_embedded_frame(&self, source: &Self::Source) -> EmbeddedFrame;
}

impl<T: SelfContainedGetEmbeddedFrame> GetEmbeddedFrame for SelfContained<T> {
    type Source = ();

    fn get_embedded_frame(&self, _source: &Self::Source) -> EmbeddedFrame {
        self.inner().self_contained_get_embedded_frame()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct IndirectEmbeddedFrame {
    offset: usize,
}

impl IndirectEmbeddedFrame {
    pub const fn new(offset: usize) -> Self {
        Self { offset }
    }

    pub const fn offset(&self) -> usize {
        self.offset
    }
}

impl GetEmbeddedFrame for IndirectEmbeddedFrame {
    type Source = [u8];

    fn get_embedded_frame(&self, source: &Self::Source) -> EmbeddedFrame {
        EmbeddedFrame::new(&source[self.offset()])
    }
}

#[macro_export]
macro_rules! embed_frame {
    ($frame_size:expr, $content:expr) => {{
        #[repr(C, align($frame_size))]
        struct Aligned<T: ?Sized>(T);

        const FRAME: &'static Aligned<[u8]> = &Aligned($content);

        $crate::EmbeddedFrame::new(FRAME.0.as_ptr())
    }};
}

// // //

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct Fill<D> {
    pub entries: Box<[FillEntry<D>]>,
}

impl<D> Fill<D> {
    pub fn depends_on_bootinfo(&self) -> bool {
        self.entries.iter().any(|entry| entry.content.is_bootinfo())
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct FillEntry<D> {
    pub range: Range<usize>,
    pub content: FillEntryContent<D>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub enum FillEntryContent<D> {
    Data(D),
    BootInfo(FillEntryContentBootInfo),
}

impl<D> FillEntryContent<D> {
    pub fn as_data(&self) -> Option<&D> {
        match self {
            Self::Data(data) => Some(data),
            _ => None,
        }
    }

    pub fn as_bootinfo(&self) -> Option<&FillEntryContentBootInfo> {
        match self {
            Self::BootInfo(info) => Some(info),
            _ => None,
        }
    }

    pub fn is_data(&self) -> bool {
        self.as_data().is_some()
    }

    pub fn is_bootinfo(&self) -> bool {
        self.as_bootinfo().is_some()
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct FillEntryContentBootInfo {
    pub id: FillEntryContentBootInfoId,
    pub offset: usize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub enum FillEntryContentBootInfoId {
    Fdt,
}

// // //

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct FileContent {
    pub file: String,
    pub file_offset: usize,
}

impl FileContent {
    pub fn with_length(&self, length: usize) -> FileContentRange {
        FileContentRange {
            file: self.file.clone(),
            file_offset: self.file_offset,
            file_length: length,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct FileContentRange {
    pub file: String,
    pub file_offset: usize,
    pub file_length: usize,
}

impl FileContentRange {
    pub fn file_range(&self) -> Range<usize> {
        self.file_offset..self.file_offset + self.file_length
    }
}

// // //

pub trait SelfContainedContent {
    fn self_contained_copy_out(&self, dst: &mut [u8]);
}

#[derive(Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct BytesContent<'a> {
    pub bytes: &'a [u8],
}

impl BytesContent<'_> {
    pub fn pack(raw_content: &[u8]) -> Vec<u8> {
        raw_content.to_vec()
    }
}

impl SelfContainedContent for BytesContent<'_> {
    fn self_contained_copy_out(&self, dst: &mut [u8]) {
        dst.copy_from_slice(self.bytes)
    }
}

impl fmt::Debug for BytesContent<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BytesContent")
            .field("bytes", &"&[...]")
            .finish()
    }
}

#[cfg(feature = "deflate")]
#[derive(Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct DeflatedBytesContent<'a> {
    pub deflated_bytes: &'a [u8],
}

#[cfg(feature = "deflate")]
impl DeflatedBytesContent<'_> {
    pub fn pack(raw_content: &[u8]) -> Vec<u8> {
        miniz_oxide::deflate::compress_to_vec(raw_content, 10)
    }
}

#[cfg(feature = "deflate")]
impl SelfContainedContent for DeflatedBytesContent<'_> {
    fn self_contained_copy_out(&self, dst: &mut [u8]) {
        let n = miniz_oxide::inflate::decompress_slice_iter_to_slice(
            dst,
            iter::once(self.deflated_bytes),
            false, // zlib_header
            true,  // ignore_adler32
        )
        .unwrap();
        assert_eq!(n, dst.len())
    }
}

#[cfg(feature = "deflate")]
impl fmt::Debug for DeflatedBytesContent<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeflatedBytesContent")
            .field("deflated_bytes", &"&[...]")
            .finish()
    }
}

// // //

pub trait Content {
    type Source: ?Sized;

    fn copy_out(&self, source: &Self::Source, dst: &mut [u8]);
}

impl<T: SelfContainedContent> Content for SelfContained<T> {
    type Source = ();

    fn copy_out(&self, _source: &Self::Source, dst: &mut [u8]) {
        self.inner().self_contained_copy_out(dst)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct IndirectBytesContent {
    pub bytes_range: Range<usize>,
}

impl Content for IndirectBytesContent {
    type Source = [u8];

    fn copy_out(&self, source: &Self::Source, dst: &mut [u8]) {
        BytesContent {
            bytes: &source[self.bytes_range.clone()],
        }
        .self_contained_copy_out(dst)
    }
}

#[cfg(feature = "deflate")]
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct IndirectDeflatedBytesContent {
    pub deflated_bytes_range: Range<usize>,
}

#[cfg(feature = "deflate")]
impl Content for IndirectDeflatedBytesContent {
    type Source = [u8];

    fn copy_out(&self, source: &Self::Source, dst: &mut [u8]) {
        DeflatedBytesContent {
            deflated_bytes: &source[self.deflated_bytes_range.clone()],
        }
        .self_contained_copy_out(dst)
    }
}
