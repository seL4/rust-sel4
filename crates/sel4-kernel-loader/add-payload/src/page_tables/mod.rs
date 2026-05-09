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

pub use glue::{Region, RegionsBuilder};
pub use scheme::{LeafDescriptor, RawDescriptor, Scheme};
pub use table::MkLeafArgs;

pub mod schemes {
    pub use super::scheme::{AArch32LeafDescriptor, AArch64LeafDescriptor, RiscVLeafDescriptor};
}
