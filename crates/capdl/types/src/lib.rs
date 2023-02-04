#![no_std]
#![feature(never_type)]
#![feature(unwrap_infallible)]
#![feature(const_trait_impl)]
#![feature(const_borrow)]

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
pub use fill::{FillEntryContentDigest, FillEntryContentFile};

#[cfg(feature = "deflate")]
pub use fill::{FillEntryContentDeflatedBytes, FillEntryContentDeflatedBytesVia};

#[cfg(feature = "sel4")]
pub use when_sel4::*;

// // // //

// pub type SpecForLoader<'a, F, N> = ConcreteSpec<'a, SliceContainer<'a>, F, N>;

// pub type SpecForLoaderWithoutDeflate<'a, N> = SpecForLoader<'a, FillEntryContentBytes<'a>, N>;

// #[cfg(feature = "deflate")]
// pub type SpecForLoaderWithDeflate<'a, N> = SpecForLoader<'a, , N>;

// #[cfg(feature = "alloc")]
// pub type SpecForBuildSystem<'a, F> = ConcreteSpec<'a, VecContainer, F, String>;
