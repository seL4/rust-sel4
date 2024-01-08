//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(never_type)]
#![feature(pointer_is_aligned)]
#![feature(proc_macro_hygiene)]
#![feature(slice_ptr_get)]
#![feature(stmt_expr_attributes)]
#![feature(strict_provenance)]
#![feature(unwrap_infallible)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

mod cap_table;
mod footprint;
mod frame_init;
mod indirect;
mod inspect;
mod object_name;
mod spec;

#[cfg(feature = "alloc")]
mod traverse;

#[cfg(feature = "std")]
mod when_std;

#[cfg(feature = "sel4")]
mod when_sel4;

pub use cap_table::{HasCapTable, PageTableEntry};
pub use footprint::Footprint;
pub use frame_init::{
    BytesContent, Content, EmbeddedFrame, Fill, FillEntry, FillEntryContent,
    FillEntryContentBootInfo, FillEntryContentBootInfoId, FrameInit, GetEmbeddedFrame,
    IndirectBytesContent, IndirectEmbeddedFrame, SelfContainedContent,
    SelfContainedGetEmbeddedFrame,
};
pub use indirect::Indirect;
pub use object_name::{
    IndirectObjectName, ObjectName, ObjectNamesLevel, SelfContainedObjectName, Unnamed,
};
pub use spec::{
    cap, object, ASIDSlotEntry, Badge, CPtr, Cap, CapSlot, CapTableEntry, IRQEntry, NamedObject,
    Object, ObjectId, Rights, Spec, TryFromCapError, TryFromObjectError, UntypedCover, Word,
};

#[cfg(feature = "alloc")]
pub use frame_init::{FileContent, FileContentRange};

#[cfg(feature = "deflate")]
pub use frame_init::{DeflatedBytesContent, IndirectDeflatedBytesContent};

#[cfg(feature = "std")]
pub use when_std::{FillMap, FillMapBuilder, InputSpec};

#[cfg(feature = "sel4")]
pub use when_sel4::*;

// // //

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SpecWithSources<'a, N: ObjectName, D: Content, M: GetEmbeddedFrame> {
    pub spec: Spec<'a, N, D, M>,
    pub object_name_source: &'a N::Source,
    pub content_source: &'a D::Source,
    pub embedded_frame_source: &'a M::Source,
}

#[cfg(feature = "deflate")]
pub type SpecWithIndirection<'a> =
    Spec<'a, Option<IndirectObjectName>, IndirectDeflatedBytesContent, IndirectEmbeddedFrame>;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SelfContained<T>(T);

impl<T> SelfContained<T> {
    pub const fn new(inner: T) -> Self {
        Self(inner)
    }

    pub const fn inner(&self) -> &T {
        &self.0
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}
