#![no_std]
#![feature(never_type)]
#![feature(proc_macro_hygiene)]
#![feature(stmt_expr_attributes)]

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

#[cfg(feature = "sel4")]
mod when_sel4;

pub use cap_table::{HasCapTable, PageTableEntry};
pub use footprint::Footprint;
pub use frame_init::{
    BytesContent, Content, Fill, FillEntry, FillEntryContent, FillEntryContentBootInfo,
    FillEntryContentBootInfoId, IndirectBytesContent, SelfContainedContent,
};
pub use indirect::Indirect;
pub use object_name::{IndirectObjectName, ObjectName, SelfContainedObjectName, Unnamed};
pub use spec::{
    cap, object, ASIDSlotEntry, Badge, CPtr, Cap, CapSlot, CapTableEntry, IRQEntry, NamedObject,
    Object, ObjectId, Rights, Spec, TryFromCapError, TryFromObjectError, Word,
};

#[cfg(feature = "alloc")]
pub use frame_init::{FileContent, FileContentRange};

#[cfg(feature = "deflate")]
pub use frame_init::{DeflatedBytesContent, IndirectDeflatedBytesContent};

#[cfg(feature = "sel4")]
pub use when_sel4::*;

// // //

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SpecWithSources<'a, N: ObjectName, F: Content> {
    pub spec: Spec<'a, N, F>,
    pub content_source: &'a F::Source,
    pub object_name_source: &'a N::Source,
}

pub type SelfContainedSpec<'a, N, F> = Spec<'a, SelfContained<N>, SelfContained<F>>;

#[derive(Debug, Clone, Eq, PartialEq)]
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
