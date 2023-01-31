#![no_std]
#![feature(never_type)]
#![feature(unwrap_infallible)]
#![feature(const_trait_impl)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::string::String;

mod cap_table;
mod container;
mod fill;
mod inspect;
mod object_name;
mod spec;
mod traverse_simple;

#[cfg(feature = "alloc")]
mod traverse;

#[cfg(feature = "sel4")]
mod when_sel4;

pub use cap_table::{CapSlot, CapTableEntry, HasCapTable, PDEntry};
pub use container::{Container, ContainerType, SliceContainer};
pub use fill::{AvailableFillEntryContent, FillEntryContentBytes};
pub use object_name::{ObjectName, Unnamed};
pub use spec::{
    cap, object, ASIDSlotEntry, Badge, CPtr, Cap, FillEntry, IRQEntry, NamedObject, Object,
    ObjectId, Rights, Spec, TryFromCapError, TryFromObjectError, Word,
};

#[cfg(feature = "alloc")]
pub use container::VecContainer;

#[cfg(feature = "alloc")]
pub use fill::{FillEntryContentDigest, FillEntryContentFile};

#[cfg(feature = "deflate")]
pub use fill::FillEntryContentDeflatedBytes;

#[cfg(feature = "sel4")]
pub use when_sel4::*;

// // //

pub type SpecForLoader<'a, S, C> = ConcreteSpec<'a, SliceContainer<'a>, C, S>;

pub type SpecForLoaderWithoutDeflate<'a, S> = SpecForLoader<'a, S, FillEntryContentBytes<'a>>;

#[cfg(feature = "deflate")]
pub type SpecForLoaderWithDeflate<'a, S> = SpecForLoader<'a, S, FillEntryContentDeflatedBytes<'a>>;

#[cfg(feature = "alloc")]
pub type SpecForBuildSystem<'a, C> = ConcreteSpec<'a, VecContainer, C, String>;

// // //

pub type ConcreteSpec<'a, T, F, N> = Spec<
    ContainerType<'a, T, ConcreteNamedObject<'a, T, F, N>>,
    ContainerType<'a, T, IRQEntry>,
    ContainerType<'a, T, ASIDSlotEntry>,
>;

pub type ConcreteNamedObject<'a, T, F, N> =
    NamedObject<N, ContainerType<'a, T, CapTableEntry>, ContainerType<'a, T, FillEntry<F>>>;

pub type ConcreteObject<'a, T, F> =
    Object<ContainerType<'a, T, CapTableEntry>, ContainerType<'a, T, FillEntry<F>>>;

pub type ConcreteCapTable<'a, T> = ContainerType<'a, T, CapTableEntry>;
pub type ConcreteFillEntries<'a, T, F> = ContainerType<'a, T, FillEntry<F>>;
