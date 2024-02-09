//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

mod embed;
mod glue;
mod regions;
mod scheme;
mod table;

pub use glue::{Region, Regions, RegionsBuilder};
pub use regions::{AbstractRegion, AbstractRegions, AbstractRegionsBuilder};
pub use scheme::{Scheme, SchemeHelpers};
pub use table::{LeafLocation, MkLeafFn, RegionContent, Table};

pub mod schemes {
    pub use crate::scheme::{AArch32, AArch64, RiscV32Sv32, RiscV64Sv39};
}
