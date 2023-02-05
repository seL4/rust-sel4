#![no_std]
#![feature(const_borrow)]
#![feature(const_trait_impl)]
#![feature(never_type)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod cap_table;
mod fill;
mod indirect;
mod inspect;
mod object_name;
mod spec;

#[cfg(feature = "alloc")]
mod traverse;

#[cfg(feature = "sel4")]
mod when_sel4;

pub use cap_table::{HasCapTable, PDEntry};
pub use fill::{AvailableFillEntryContent, AvailableFillEntryContentVia, FillEntryContentBytes};
pub use indirect::Indirect;
pub use object_name::{ObjectName, Unnamed};
pub use spec::{
    cap, object, ASIDSlotEntry, Badge, CPtr, Cap, CapSlot, CapTableEntry, FillEntry,
    FillEntryContent, FillEntryContentBootInfo, FillEntryContentBootInfoId, IRQEntry, NamedObject,
    Object, ObjectId, Rights, Spec, TryFromCapError, TryFromObjectError, Word,
};

#[cfg(feature = "alloc")]
pub use fill::FillEntryContentFile;

#[cfg(feature = "deflate")]
pub use fill::{FillEntryContentDeflatedBytes, FillEntryContentDeflatedBytesVia};

#[cfg(feature = "sel4")]
pub use when_sel4::*;
