//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::ops::Range;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub enum FrameInit {
    Fill(Fill<Content>),
    Embedded(EmbeddedFrameIndex),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct EmbeddedFrameIndex {
    pub index: u64,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct Fill<D> {
    pub entries: Vec<FillEntry<D>>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct FillEntry<D> {
    pub range: Range<u64>,
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
    pub offset: u64,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub enum FillEntryContentBootInfoId {
    Fdt,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct FillEntryContentFileOffset {
    pub file: String,
    pub file_offset: u64,
}

#[derive(Debug, Clone, Eq, PartialEq, rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub enum Content {
    Bytes(BytesContent),
    DeflatedBytes(DeflatedBytesContent),
}

impl Content {
    pub fn copy_out(&self, dst: &mut [u8]) {
        match self {
            Self::Bytes(bytes) => bytes.copy_out(dst),
            Self::DeflatedBytes(deflated_bytes) => deflated_bytes.copy_out(dst),
        }
    }
}

impl ArchivedContent {
    pub fn copy_out(&self, dst: &mut [u8]) {
        match self {
            Self::Bytes(bytes) => bytes.copy_out(dst),
            Self::DeflatedBytes(deflated_bytes) => deflated_bytes.copy_out(dst),
        }
    }
}

#[derive(Clone, Eq, PartialEq, rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct BytesContent {
    pub bytes: Vec<u8>,
}

impl fmt::Debug for BytesContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BytesContent")
            .field("bytes", &Omitted)
            .finish()
    }
}

impl BytesContent {
    pub fn pack(raw_content: &[u8]) -> Self {
        Self {
            bytes: raw_content.to_vec(),
        }
    }
}

impl BytesContent {
    pub fn copy_out(&self, dst: &mut [u8]) {
        dst.copy_from_slice(&self.bytes)
    }
}

impl ArchivedBytesContent {
    pub fn copy_out(&self, dst: &mut [u8]) {
        dst.copy_from_slice(&self.bytes)
    }
}

#[derive(Clone, Eq, PartialEq, rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct DeflatedBytesContent {
    pub deflated_bytes: Vec<u8>,
}

impl fmt::Debug for DeflatedBytesContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeflatedBytesContent")
            .field("deflated_bytes", &Omitted)
            .finish()
    }
}

#[cfg(feature = "deflate")]
impl DeflatedBytesContent {
    pub fn pack(raw_content: &[u8]) -> Self {
        Self {
            deflated_bytes: miniz_oxide::deflate::compress_to_vec(raw_content, 10),
        }
    }
}

impl DeflatedBytesContent {
    pub fn copy_out(&self, dst: &mut [u8]) {
        copy_out_deflated(&self.deflated_bytes, dst)
    }
}

impl ArchivedDeflatedBytesContent {
    pub fn copy_out(&self, dst: &mut [u8]) {
        copy_out_deflated(&self.deflated_bytes, dst)
    }
}

#[cfg(feature = "deflate")]
fn copy_out_deflated(deflated_src: &[u8], dst: &mut [u8]) {
    let n = miniz_oxide::inflate::decompress_slice_iter_to_slice(
        dst,
        core::iter::once(deflated_src),
        false, // zlib_header
        true,  // ignore_adler32
    )
    .unwrap();
    assert_eq!(n, dst.len())
}

#[cfg(not(feature = "deflate"))]
fn copy_out_deflated(_deflated_src: &[u8], _dst: &mut [u8]) {
    panic!("found deflated data but \"deflate\" feature is not enabled")
}

// impl Debug helper
struct Omitted;

impl fmt::Debug for Omitted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<omitted>")
    }
}
