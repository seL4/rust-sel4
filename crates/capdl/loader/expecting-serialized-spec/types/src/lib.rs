#![no_std]

extern crate alloc;

use capdl_types::*;

pub type SpecForSerialization<'a> =
    Spec<'a, Option<IndirectObjectName>, IndirectDeflatedBytesContent>;

pub type SpecWithSourcesForSerialization<'a> =
    SpecWithSources<'a, Option<IndirectObjectName>, IndirectDeflatedBytesContent>;
