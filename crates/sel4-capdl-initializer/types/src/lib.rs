//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

extern crate alloc;

use rkyv::Archive;
use rkyv::rancor;
use rkyv::util::AlignedVec;

mod cap_table;
mod frame_init;
mod spec;

#[cfg(feature = "fill-utils")]
mod fill_utils;

#[cfg(feature = "sel4")]
mod when_sel4;

pub use cap_table::{HasArchivedCapTable, HasCapTable};
pub use frame_init::*;
pub use spec::*;

#[cfg(feature = "sel4")]
pub use when_sel4::*;

pub type InputSpec = Spec<Fill<FillEntryContentFileOffset>>;

pub type SpecForInitializer = Spec<FrameInit>;

impl SpecForInitializer {
    pub fn to_bytes(&self) -> Result<AlignedVec, rancor::Error> {
        rkyv::to_bytes(self)
    }

    pub fn access(buf: &[u8]) -> Result<&<Self as Archive>::Archived, rancor::Error> {
        rkyv::access(buf)
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn access_unchecked(buf: &[u8]) -> &<Self as Archive>::Archived {
        unsafe { rkyv::access_unchecked(buf) }
    }
}
