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

pub(crate) use glue::{Region, RegionsBuilder};
pub(crate) use scheme::{LeafDescriptor, RawDescriptor, Scheme};
pub(crate) use table::MkLeafArgs;

pub(crate) mod schemes {
    pub(crate) use super::scheme::{
        AArch32LeafDescriptor, AArch64LeafDescriptor, RiscVLeafDescriptor,
    };
}
