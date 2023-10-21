//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::ops::Range;

use crate::regions::{AbstractRegion, AbstractRegions, AbstractRegionsBuilder};
use crate::scheme::{Scheme, SchemeHelpers};
use crate::table::{LeafLocation, MkLeafFn, RegionContent, Table};

pub type Region<T> = AbstractRegion<Option<RegionContent<T>>>;
pub type Regions<T> = AbstractRegions<Option<RegionContent<T>>>;
pub type RegionsBuilder<T> = AbstractRegionsBuilder<Option<RegionContent<T>>>;

impl<T: Scheme> RegionsBuilder<T> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self::new_with_background(Region::invalid(SchemeHelpers::<T>::virt_bounds()))
    }
}

impl<T: Scheme> Regions<T> {
    pub fn construct_table(&self) -> Table<T> {
        Table::construct(self)
    }
}

impl<T: Scheme> Region<T> {
    pub fn valid(range: Range<u64>, mk_leaf: impl MkLeafFn<T> + 'static) -> Self {
        Self {
            range,
            content: Some(RegionContent::new(mk_leaf)),
        }
    }

    pub fn invalid(range: Range<u64>) -> Self {
        Self {
            range,
            content: None,
        }
    }
}

impl LeafLocation {
    pub fn map<T: Scheme>(&self, vaddr_to_paddr: impl FnOnce(u64) -> u64) -> T::LeafDescriptor {
        SchemeHelpers::<T>::leaf_descriptor_from_paddr_with_check(
            (vaddr_to_paddr)(self.vaddr()),
            self.level(),
        )
    }

    pub fn map_identity<T: Scheme>(&self) -> T::LeafDescriptor {
        self.map::<T>(|vaddr| vaddr)
    }
}
